use ares_core::{AresError, ProjectId, NodeId};
use ares_store::repositories::graph::SqliteGraphRepository;
use ares_store::repositories::memory::SqliteMemoryRepository;
use std::sync::Arc;
use std::time::Instant;

use crate::impact::ImpactAnalyzer;
use crate::models::{ContextBundle, ContextPack, ContextMetrics, FileExplanation, RepositoryInsight};
use crate::query::{IntentDetector, QueryParser, QueryIntent};
use crate::ranking::{DistanceScorer, HybridRanker, RankingStrategy, RecencyScorer};
use crate::retrieval::{BundleBuilder, GraphRetriever, MemoryRetriever, SummaryRetriever};
use crate::traversal::{ArchitectureTraverser, DependencyTraverser, NeighborTraverser, ShortestPathTraverser, TraversalConfig};

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
        
        let intent = self.intent_detector.detect(query);
        let targets = self.query_parser.extract_targets(query);
        
        let mut bundle_builder = BundleBuilder::new();
        bundle_builder.set_query(query.to_string());
        bundle_builder.set_intent(intent.clone());
        
        // Retrieval Time Start
        let r_start = Instant::now();
        
        // 1. Target resolution
        let mut seed_nodes = Vec::new();
        if !targets.is_empty() {
            let nodes = self.graph_retriever.resolve_files(&self.project_id, &targets).await?;
            seed_nodes = nodes;
        }

        let retrieval_time = r_start.elapsed().as_millis() as u64;

        // Traversal Time Start
        let t_start = Instant::now();

        // 2. Intent-specific Traversal & Impact
        match intent {
            QueryIntent::ChangeImpact => {
                for node in &seed_nodes {
                    let report = self.impact_analyzer.analyze(&self.project_id, &node.id).await?;
                    bundle_builder.add_impact_report(report);
                }
            }
            QueryIntent::DependencyTrace => {
                for node in &seed_nodes {
                    let trace = self.dependency_traverser.trace_dependents(&self.project_id, &node.id).await?;
                    bundle_builder.add_dependency_trace(trace);
                }
            }
            QueryIntent::RepositoryOverview | QueryIntent::ArchitectureQuery => {
                if let Some(summary_content) = self.summary_retriever.fetch_latest_summary(&self.project_id).await? {
                    bundle_builder.add_insight(RepositoryInsight {
                        summary: summary_content,
                    });
                }
            }
            QueryIntent::FileExplanation => {
                for node in &seed_nodes {
                    let deps = self.dependency_traverser.trace_dependents(&self.project_id, &node.id).await?;
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

        // Ranking Time Start
        let rank_start = Instant::now();
        
        // For demonstration, rank all seeds. A full impl would rank all traversed nodes.
        let nodes_to_rank: Vec<_> = seed_nodes.into_iter().map(|n| (n, 0)).collect();
        let ranked_nodes = self.ranker.rank(nodes_to_rank);
        
        let ranking_time = rank_start.elapsed().as_millis() as u64;
        let _total_time = start.elapsed().as_millis() as u64;

        let metrics = ContextMetrics {
            retrieval_time_ms: retrieval_time,
            traversal_time_ms: traversal_time,
            ranking_time_ms: ranking_time,
            nodes_examined: 0, // Track properly during traversal later
            nodes_returned: ranked_nodes.len(),
        };

        bundle_builder.set_ranked_nodes(ranked_nodes.into_iter().map(|rn| rn.node).collect());
        bundle_builder.set_metrics(metrics);

        Ok(bundle_builder.build())
    }

    // Direct API hooks
    
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
