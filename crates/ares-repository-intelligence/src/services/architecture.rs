use crate::engines::graph::models::GraphPayload;
use ares_core::AresError;
use ares_core::NodeType;
use ares_core::ProjectId;
use ares_core::{GraphEdge, GraphNode};
use ares_store::repositories::graph::{
    GraphQueryContext, LayoutHint, SqliteGraphRepository, WorkspaceContext,
};
use ares_store::Store;
use std::collections::{HashMap, HashSet};

pub struct ArchitectureService {
    store: Store,
}

impl ArchitectureService {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn generate_architectural_seed(
        &self,
        workspace_name: &str,
        project_id_str: &str,
        max_nodes: usize,
    ) -> Result<GraphPayload, AresError> {
        let repo = SqliteGraphRepository::new(self.store.clone());
        let project_id = ProjectId::from(project_id_str);

        let query_context = GraphQueryContext {
            workspace: WorkspaceContext {
                workspace_path: std::path::PathBuf::from(workspace_name),
                workspace_name: workspace_name.to_string(),
                repositories: vec![project_id.clone()],
            },
            repository: project_id.clone(),
            max_depth: 3,
            max_nodes,
            edge_filters: vec![],
            node_filters: vec![], // We'll filter in Rust for now
            layout_hint: LayoutHint::Hierarchical,
        };

        // 1. Fetch BFS root graph (we can fetch more nodes than we need, then rank them)
        let (nodes, edges) = repo.get_graph_root(&query_context)?;

        // 2. Ranking logic
        let ranked_nodes = self.rank_nodes(&nodes, &edges);

        // 3. Take top N
        let mut top_node_ids: HashSet<String> = ranked_nodes
            .into_iter()
            .take(max_nodes)
            .map(|n| n.id.as_str().to_string())
            .collect();

        // Always include project/repository root if present
        if let Some(root) = nodes.iter().find(|n| n.node_type == NodeType::Project) {
            top_node_ids.insert(root.id.as_str().to_string());
        }

        // 4. Filter nodes and edges to match top N
        let filtered_nodes = nodes
            .into_iter()
            .filter(|n| top_node_ids.contains(n.id.as_str()))
            .collect();
        let filtered_edges = edges
            .into_iter()
            .filter(|e| {
                top_node_ids.contains(e.from_node_id.as_str())
                    && top_node_ids.contains(e.to_node_id.as_str())
            })
            .collect();

        Ok(GraphPayload {
            nodes: filtered_nodes,
            edges: filtered_edges,
        })
    }

    fn rank_nodes<'a>(&self, nodes: &'a [GraphNode], edges: &'a [GraphEdge]) -> Vec<&'a GraphNode> {
        let mut degree_map: HashMap<&str, usize> = HashMap::new();
        for edge in edges {
            *degree_map.entry(edge.from_node_id.as_str()).or_insert(0) += 1;
            *degree_map.entry(edge.to_node_id.as_str()).or_insert(0) += 1;
        }

        let mut scored_nodes: Vec<(&GraphNode, usize)> = nodes
            .iter()
            .map(|n| {
                let mut score = 0;

                // Prioritize folders/crates/modules
                match n.node_type {
                    NodeType::Project => score += 1000,
                    NodeType::Folder => score += 500,
                    NodeType::Module => score += 300,
                    NodeType::File => score += 100,
                    _ => {}
                }

                // Add degree to score
                let degree = degree_map.get(n.id.as_str()).copied().unwrap_or(0);
                score += degree * 10;

                (n, score)
            })
            .collect();

        // Sort by score descending
        scored_nodes.sort_by(|a, b| b.1.cmp(&a.1));

        scored_nodes.into_iter().map(|(n, _)| n).collect()
    }
}
