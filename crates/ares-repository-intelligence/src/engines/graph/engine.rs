use super::models::{GraphPayload, GraphStatistics};
use ares_core::id::NodeId;
use ares_core::AresError;
use ares_store::Store;

pub struct RepositoryGraphEngine {
    pub store: Store,
}

impl RepositoryGraphEngine {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
    pub async fn graph_statistics(store: &Store) -> Result<GraphStatistics, AresError> {
        let stats = store.graph_metrics()?;
        Ok(GraphStatistics {
            nodes: stats.total_nodes,
            edges: stats.total_edges,
            types: stats.node_type_counts,
        })
    }

    pub async fn graph_root(
        store: &Store,
        context: &RepositoryContext,
    ) -> Result<GraphPayload, AresError> {
        let query_context = GraphQueryContext {
            workspace: ares_store::repositories::graph::WorkspaceContext {
                workspace_path: std::path::PathBuf::from("/workspace"),
                workspace_name: "workspace".to_string(),
                repositories: vec![ares_core::ProjectId::from(
                    context.workspace.workspace_id.as_str(),
                )],
            },
            repository: ares_core::ProjectId::from(context.workspace.workspace_id.as_str()),
            max_depth: 3,
            max_nodes: 100,
            edge_filters: vec![],
            node_filters: vec![],
            layout_hint: ares_store::repositories::graph::LayoutHint::Hierarchical,
        };
        let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store.clone());
        let (nodes, edges) = repo.get_graph_root(&query_context)?;

        tracing::info!("Nodes = {}", nodes.len());
        tracing::info!("Edges = {}", edges.len());
        if let Some(first) = nodes.first() {
            tracing::info!("{:#?}", first);
        }

        Ok(GraphPayload { nodes, edges })
    }

    pub async fn graph_neighbors(
        store: &Store,
        node_id: &NodeId,
    ) -> Result<GraphPayload, AresError> {
        let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store.clone());
        let (nodes, edges) = repo.get_graph_neighbors(node_id, 1)?;
        Ok(GraphPayload { nodes, edges })
    }

    pub async fn graph_search(store: &Store, query: &str) -> Result<GraphPayload, AresError> {
        let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store.clone());
        let (nodes, edges) = repo.search_graph_nodes(query)?;
        Ok(GraphPayload { nodes, edges })
    }

    pub async fn graph_shortest_path(
        store: &Store,
        from: &NodeId,
        to: &NodeId,
    ) -> Result<GraphPayload, AresError> {
        let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store.clone());
        let (nodes, edges) = repo.get_shortest_path(from, to)?;
        Ok(GraphPayload { nodes, edges })
    }

    pub async fn graph_subgraph(
        store: &Store,
        node_id: &NodeId,
        depth: usize,
    ) -> Result<GraphPayload, AresError> {
        let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store.clone());
        let (nodes, edges) = repo.get_graph_neighbors(node_id, depth)?;
        Ok(GraphPayload { nodes, edges })
    }

    pub async fn graph_metadata(
        store: &Store,
        node_id: &NodeId,
    ) -> Result<super::models::GraphNodeDetails, AresError> {
        let repo = ares_store::repositories::graph::SqliteGraphRepository::new(store.clone());
        let node = repo
            .get_node(node_id)
            .map_err(|e| ares_core::AresError::Database(e.to_string()))?
            .ok_or_else(|| ares_core::AresError::not_found("node", node_id.as_str()))?;

        // Extract neighbors
        let mut edges = repo
            .get_edges_from(node_id)
            .map_err(|e| ares_core::AresError::Database(e.to_string()))?;
        let mut to_edges = repo
            .get_edges_to(node_id)
            .map_err(|e| ares_core::AresError::Database(e.to_string()))?;
        edges.append(&mut to_edges);

        let mut calls = Vec::new();
        let mut called_by = Vec::new();
        let mut imports = Vec::new();
        let mut imported_by = Vec::new();
        let mut depends_on = Vec::new();

        for edge in edges {
            let edge_type_str = format!("{:?}", edge.edge_type);
            let mut cit = crate::core::response::Citation {
                kind: edge_type_str.clone(),
                id: if edge.from_node_id == *node_id {
                    edge.to_node_id.as_str().to_string()
                } else {
                    edge.from_node_id.as_str().to_string()
                },
                title: "".to_string(), // we can fetch titles in a real implementation
                location: None,
            };
            cit.title = cit.id.clone(); // fallback

            if edge.from_node_id == *node_id {
                match edge_type_str.as_str() {
                    "Calls" => calls.push(cit),
                    "Imports" => imports.push(cit),
                    "DependsOn" => depends_on.push(cit),
                    _ => {}
                }
            } else {
                match edge_type_str.as_str() {
                    "Calls" => called_by.push(cit),
                    "Imports" => imported_by.push(cit),
                    _ => {}
                }
            }
        }

        Ok(super::models::GraphNodeDetails {
            overview: super::models::NodeOverview {
                node_id: node.id.as_str().to_string(),
                node_type: format!("{:?}", node.node_type),
                language: node
                    .properties
                    .get("language")
                    .and_then(|v| v.as_str().map(|s| s.to_string())),
                repository: None,
                loc: None,
                module: None,
                namespace: None,
            },
            health: super::models::NodeHealth {
                confidence: 1.0,
                owner_confidence: 1.0,
                last_modified_days_ago: None,
                missing_requirements: true,
                missing_decisions: true,
                is_orphan: called_by.is_empty(),
                drift: false,
                test_coverage: None,
            },
            ownership: super::models::NodeOwnership {
                primary_owner: None,
                git_authors: Vec::new(),
                team: None,
            },
            relationships: super::models::NodeRelationships {
                calls,
                called_by,
                imports,
                imported_by,
                depends_on,
                required_by: Vec::new(),
                implements: Vec::new(),
                inherited_by: Vec::new(),
            },
            history: super::models::NodeHistory {
                created_at: None,
                last_modified: None,
                commits: 0,
            },
            architecture: super::models::NodeArchitecture {
                requirements: Vec::new(),
                adrs: Vec::new(),
                decisions: Vec::new(),
            },
            analysis: super::models::NodeAnalysis {
                knowledge_debt: None,
                risk_level: None,
            },
            evidence: super::models::NodeEvidence {
                files: Vec::new(),
                functions: Vec::new(),
                commits: Vec::new(),
            },
        })
    }
}

use crate::core::capabilities::Capability;
use crate::core::context::RepositoryContext;
use crate::core::engine::{
    Artifact, EngineDescriptor, EngineExecutionResult, EngineId, EngineInput, RepositoryEngine,
};
use crate::core::errors::{EngineError, EngineResult};
use crate::core::evidence::{EvidenceBundle, GraphEvidence};
use crate::core::metadata::ExecutionMetadata;
use ares_store::repositories::graph::GraphQueryContext;

#[async_trait::async_trait]
impl RepositoryEngine for RepositoryGraphEngine {
    fn descriptor(&self) -> EngineDescriptor {
        EngineDescriptor {
            id: EngineId::Graph,
            version: "0.1.0".to_string(),
            capabilities: vec![Capability::GraphSearch],
            planner_api_version: 1,
        }
    }

    async fn execute(
        &self,
        context: &RepositoryContext,
        input: EngineInput,
    ) -> EngineResult<EngineExecutionResult> {
        let start = std::time::Instant::now();

        let payload = match input {
            EngineInput::NodeId(id) => {
                let node_id = NodeId::from(id.as_str());
                Self::graph_neighbors(&self.store, &node_id).await
            }
            EngineInput::Query(q) => Self::graph_search(&self.store, &q).await,
            EngineInput::None => Self::graph_root(&self.store, context).await,
        }
        .map_err(|e| EngineError::ExecutionError(e.to_string()))?;

        let json_content = serde_json::to_string(&payload)
            .map_err(|e| EngineError::InternalError(e.to_string()))?;

        let artifacts = vec![Artifact {
            name: "graph_payload.json".to_string(),
            format: "JSON".to_string(),
            content: json_content,
        }];

        let evidence = EvidenceBundle {
            graph: Some(GraphEvidence {
                nodes: payload.nodes.iter().map(|n| n.id.to_string()).collect(),
                edges: payload.edges.iter().map(|e| e.id.to_string()).collect(),
                paths: vec![],
            }),
            ..EvidenceBundle::default()
        };

        let metadata = ExecutionMetadata {
            engine: "RepositoryGraphEngine".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
            cache_hit: false,
            confidence: 1.0,
            errors: vec![],
            warnings: vec![],
            retry_count: 0,
            sources_used: vec!["RepositoryGraphEngine".to_string()],
        };

        Ok(EngineExecutionResult {
            descriptor: self.descriptor(),
            engine_id: EngineId::Graph,
            capability: Capability::GraphSearch,
            evidence,
            metadata,
            diagnostics: std::collections::HashMap::new(),
            warnings: vec![],
            errors: vec![],
            artifacts,
        })
    }
}
