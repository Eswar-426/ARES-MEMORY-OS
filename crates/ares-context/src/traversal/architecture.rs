use super::TraversalConfig;
use ares_core::{AresError, EdgeType, NodeId};
use ares_store::repositories::graph::SqliteGraphRepository;
use std::sync::Arc;

// A simple structure to represent architectural chains
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchitecturePath {
    pub nodes: Vec<ares_core::GraphNode>,
}

pub struct ArchitectureTraverser {
    repo: Arc<SqliteGraphRepository>,
    config: TraversalConfig,
}

impl ArchitectureTraverser {
    pub fn new(repo: Arc<SqliteGraphRepository>, config: TraversalConfig) -> Self {
        Self { repo, config }
    }

    /// Traces from a file/function up to its parent Folders and Project node
    pub async fn get_architecture_chain(
        &self,
        _project_id: &ares_core::ProjectId,
        node_id: &NodeId,
    ) -> Result<ArchitecturePath, AresError> {
        let mut path = Vec::new();
        let mut current_id = node_id.clone();
        let mut depth = 0;

        while depth < self.config.max_depth {
            if let Some(node) = self.repo.get_node(&current_id)? {
                path.push(node);
            } else {
                break;
            }

            // Find parent using `Contains` edges where target = current_id
            let incoming = self.repo.get_edges_to(&current_id)?;
            let mut found_parent = false;
            for edge in incoming {
                if edge.edge_type == EdgeType::Contains
                    && edge.to_node_id.as_str() == current_id.as_str()
                {
                    current_id = NodeId::from(edge.from_node_id.as_str());
                    found_parent = true;
                    break;
                }
            }

            if !found_parent {
                break;
            }
            depth += 1;
        }

        Ok(ArchitecturePath { nodes: path })
    }
}
