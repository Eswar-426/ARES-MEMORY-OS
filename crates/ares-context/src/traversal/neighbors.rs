use super::TraversalConfig;
use ares_core::{AresError, GraphNode, NodeId};
use ares_store::repositories::graph::SqliteGraphRepository;
use std::sync::Arc;

pub struct NeighborTraverser {
    repo: Arc<SqliteGraphRepository>,
    config: TraversalConfig,
}

impl NeighborTraverser {
    pub fn new(repo: Arc<SqliteGraphRepository>, config: TraversalConfig) -> Self {
        Self { repo, config }
    }

    /// Fetches immediate neighbors for a node
    pub async fn get_neighbors(
        &self,
        _project_id: &ares_core::ProjectId,
        node_id: &NodeId,
    ) -> Result<Vec<GraphNode>, AresError> {
        let mut edges = self.repo.get_edges_from(node_id)?;
        if let Ok(mut incoming) = self.repo.get_edges_to(node_id) {
            edges.append(&mut incoming);
        }
        let mut neighbors = Vec::new();

        for edge in edges.into_iter().take(self.config.max_neighbors) {
            let neighbor_id = if edge.from_node_id.as_str() == node_id.as_str() {
                NodeId::from(edge.to_node_id.as_str())
            } else {
                NodeId::from(edge.from_node_id.as_str())
            };

            if let Some(node) = self.repo.get_node(&neighbor_id)? {
                neighbors.push(node);
            }

            if neighbors.len() >= self.config.max_results {
                break;
            }
        }

        Ok(neighbors)
    }
}
