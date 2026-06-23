use super::TraversalConfig;
use crate::models::DependencyTrace;
use ares_core::{AresError, EdgeType, NodeId};
use ares_store::repositories::graph::SqliteGraphRepository;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

pub struct DependencyTraverser {
    repo: Arc<SqliteGraphRepository>,
    config: TraversalConfig,
}

impl DependencyTraverser {
    pub fn new(repo: Arc<SqliteGraphRepository>, config: TraversalConfig) -> Self {
        Self { repo, config }
    }

    /// Finds all downstream dependents (things that depend on `node_id`) up to max_depth.
    pub async fn trace_dependents(
        &self,
        _project_id: &ares_core::ProjectId,
        node_id: &NodeId,
    ) -> Result<DependencyTrace, AresError> {
        let mut path = Vec::new();
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        queue.push_back((node_id.clone(), 0));
        visited.insert(node_id.clone());

        let mut max_depth_reached = 0;

        while let Some((current_id, depth)) = queue.pop_front() {
            if depth > self.config.max_depth {
                continue;
            }
            if path.len() >= self.config.max_results {
                break;
            }
            if depth > max_depth_reached {
                max_depth_reached = depth;
            }

            if let Some(node) = self.repo.get_node(&current_id)? {
                path.push(node);
            }

            // Find dependents (edges where target = current_id and edge_type = Imports/Depends)
            // Note: In our current schema, `ares_store` might not have `get_incoming_edges`.
            // We'll simulate this by grabbing all edges or assume we have an appropriate method.
            let incoming = self.repo.get_edges_to(&current_id)?;

            let mut neighbors_added = 0;
            for edge in incoming {
                if edge.edge_type == EdgeType::Imports || edge.edge_type == EdgeType::Calls {
                    // If the current node is the target, then edge.source is the dependent
                    if edge.to_node_id.as_str() == current_id.as_str() {
                        let source_id = NodeId::from(edge.from_node_id.as_str());
                        if !visited.contains(&source_id)
                            && neighbors_added < self.config.max_neighbors
                        {
                            visited.insert(source_id.clone());
                            queue.push_back((source_id, depth + 1));
                            neighbors_added += 1;
                        }
                    }
                }
            }
        }

        Ok(DependencyTrace {
            target: node_id.as_str().to_string(),
            path,
            depth: max_depth_reached,
        })
    }
}
