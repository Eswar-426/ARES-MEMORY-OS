use ares_core::{AresError, ProjectId};
use ares_store::repositories::graph::SqliteGraphRepository;
use ares_store::repositories::memory::SqliteMemoryRepository;
use std::sync::Arc;
use std::time::Instant;

use crate::impact::ImpactAnalyzer;
use crate::models::{ContextBundle, ContextMetrics, FileExplanation, RepositoryInsight};
use crate::query::{IntentDetector, QueryIntent, QueryParser};
use crate::ranking::{HybridRanker, RankingStrategy};
use crate::retrieval::{BundleBuilder, GraphRetriever, MemoryRetriever, SummaryRetriever};
use crate::traversal::{
    ArchitectureTraverser, DependencyTraverser, NeighborTraverser, ShortestPathTraverser,
    TraversalConfig,
};

pub struct ContextEngine {
    project_id: ProjectId,
    config: TraversalConfig,

    // Repos
    graph_repo: Arc<SqliteGraphRepository>,
    memory_repo: Arc<SqliteMemoryRepository>,

    // Sub-components
    intent_detector: IntentDetector,
    query_parser: QueryParser,

    graph_retriever: GraphRetriever,
    memory_retriever: MemoryRetriever,
    summary_retriever: SummaryRetriever,

    dependency_traverser: DependencyTraverser,
    architecture_traverser: ArchitectureTraverser,
    neighbor_traverser: NeighborTraverser,
    shortest_path_traverser: ShortestPathTraverser,

    impact_analyzer: ImpactAnalyzer,

    ranker: HybridRanker,
}

impl ContextEngine {
    pub fn new(
        project_id: ProjectId,
        graph_repo: Arc<SqliteGraphRepository>,
        memory_repo: Arc<SqliteMemoryRepository>,
        config: TraversalConfig,
    ) -> Self {
        let now = ares_core::types::event::now_micros();

        Self {
            project_id,
            config: config.clone(),

            graph_repo: graph_repo.clone(),
            memory_repo: memory_repo.clone(),

            intent_detector: IntentDetector::new(),
            query_parser: QueryParser::new(),

            graph_retriever: GraphRetriever::new(graph_repo.clone()),
            memory_retriever: MemoryRetriever::new(memory_repo.clone()),
            summary_retriever: SummaryRetriever::new(memory_repo.clone()),

            dependency_traverser: DependencyTraverser::new(graph_repo.clone(), config.clone()),
            architecture_traverser: ArchitectureTraverser::new(graph_repo.clone(), config.clone()),
            neighbor_traverser: NeighborTraverser::new(graph_repo.clone(), config.clone()),
            shortest_path_traverser: ShortestPathTraverser::new(graph_repo.clone(), config.clone()),

            impact_analyzer: ImpactAnalyzer::new(graph_repo.clone(), config.clone()),

            ranker: HybridRanker::new(now, 0.7, 0.3),
        }
    }

    /// Primary entry point: Resolves a query to a ContextBundle
    pub async fn resolve_query(&self, query: &str) -> Result<ContextBundle, AresError> {
        let start = Instant::now();

        let mut intent = self.intent_detector.detect(query);
        match intent {
            QueryIntent::ChangeImpact => intent = QueryIntent::ImpactAnalysis,
            QueryIntent::DeadCodeDiscovery => intent = QueryIntent::DeadCodeQuery,
            _ => {}
        }
        let targets = self.query_parser.extract_targets(query);

        let mut bundle_builder = BundleBuilder::new();
        bundle_builder.set_query(query.to_string());
        bundle_builder.set_intent(intent.clone());

        // Retrieval Time Start
        let r_start = Instant::now();

        // 1. Target resolution
        let mut seed_nodes = Vec::new();
        if !targets.is_empty() {
            let nodes = self
                .graph_retriever
                .resolve_files(&self.project_id, &targets)
                .await?;
            seed_nodes = nodes;
        }

        let retrieval_time = r_start.elapsed().as_millis() as u64;

        // Traversal Time Start
        let t_start = Instant::now();

        // 2. Intent-specific Traversal & Impact
        match intent {
            QueryIntent::ImpactAnalysis => {
                for node in &seed_nodes {
                    let report = self
                        .impact_analyzer
                        .analyze(&self.project_id, &node.id)
                        .await?;
                    bundle_builder.add_impact_report(report);
                }
            }
            QueryIntent::DependencyTrace => {
                for node in &seed_nodes {
                    let trace = self
                        .dependency_traverser
                        .trace_dependents(&self.project_id, &node.id)
                        .await?;
                    bundle_builder.add_dependency_trace(trace);
                }
            }
            QueryIntent::RepositoryOverview | QueryIntent::ArchitectureQuery => {
                if let Some(summary_content) = self
                    .summary_retriever
                    .fetch_latest_summary(&self.project_id)
                    .await?
                {
                    bundle_builder.add_insight(RepositoryInsight {
                        summary: summary_content,
                    });
                }
            }
            QueryIntent::FileExplanation => {
                for node in &seed_nodes {
                    let deps = self
                        .dependency_traverser
                        .trace_dependents(&self.project_id, &node.id)
                        .await?;
                    let explanation = FileExplanation {
                        file_path: node.label.clone(),
                        definitions: vec![], // In a full impl, use query on Defines edge
                        dependencies: deps.path.into_iter().map(|n| n.label).collect(),
                        related_components: vec![],
                    };
                    bundle_builder.add_explanation(explanation);
                }
            }
            _ => {
                // Fallback: just return the nodes
                bundle_builder.add_target_nodes(seed_nodes.clone());
            }
        }

        // Add seeds unconditionally
        bundle_builder.add_target_nodes(seed_nodes.clone());

        let traversal_time = t_start.elapsed().as_millis() as u64;

        let rank_start = Instant::now();

        // Compute seed_ids before moving seed_nodes
        let seed_ids: Vec<_> = seed_nodes.iter().map(|n| n.id.clone()).collect();

        // For demonstration, rank all seeds. A full impl would rank all traversed nodes.
        let nodes_to_rank: Vec<_> = seed_nodes.into_iter().map(|n| (n, 0)).collect();
        let ranked_nodes = self.ranker.rank(nodes_to_rank);

        let ranking_time = rank_start.elapsed().as_millis() as u64;
        let total_time = start.elapsed().as_millis() as u64;

        let reachable_nodes = self
            .graph_retriever
            .count_reachable_nodes(&seed_ids, 3)
            .await
            .unwrap_or(seed_ids.len());

        let metrics = ContextMetrics {
            retrieval_time_ms: retrieval_time,
            traversal_time_ms: traversal_time,
            ranking_time_ms: ranking_time,
            nodes_examined: reachable_nodes,
            nodes_returned: ranked_nodes.len(),
            ..Default::default()
        };

        bundle_builder.set_ranked_nodes(ranked_nodes.into_iter().map(|rn| rn.node).collect());
        bundle_builder.set_metrics(metrics);
        bundle_builder.set_reachable_nodes(reachable_nodes);

        let retrieved_nodes_count = bundle_builder.bundle.ranked_nodes.len();
        let graph_coverage_score = if reachable_nodes > 0 {
            retrieved_nodes_count as f64 / reachable_nodes as f64
        } else {
            1.0
        };

        let audit = crate::models::metrics::RetrievalAudit {
            query: query.to_string(),
            seed_nodes: seed_ids.len(),
            retrieved_nodes: retrieved_nodes_count,
            reachable_nodes,
            retrieval_depth: 3,
            graph_coverage_score,
            retrieval_latency_ms: total_time,
        };

        let mut out_dir = std::path::PathBuf::from("artifacts");
        out_dir.push("telemetry");
        out_dir.push("retrieval_audit");
        let _ = std::fs::create_dir_all(&out_dir);
        let _hash = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        query.hash(&mut hasher);
        let query_hash = hasher.finish();

        let out_file = out_dir.join(format!("{}.json", query_hash));
        let _ = std::fs::write(
            &out_file,
            serde_json::to_string_pretty(&audit).unwrap_or_default(),
        );

        Ok(bundle_builder.build())
    }

    pub async fn explain_file(&self, file_path: &str) -> Result<ContextBundle, AresError> {
        self.resolve_query(&format!("explain {}", file_path)).await
    }

    pub async fn trace_dependencies(&self, component: &str) -> Result<ContextBundle, AresError> {
        self.resolve_query(&format!("trace {}", component)).await
    }

    pub async fn analyze_impact(&self, component: &str) -> Result<ContextBundle, AresError> {
        self.resolve_query(&format!("impact {}", component)).await
    }
}
