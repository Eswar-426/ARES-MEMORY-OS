use super::TraversalConfig;
use ares_core::{AresError, NodeId};
use ares_store::repositories::graph::SqliteGraphRepository;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

pub struct ShortestPathTraverser {
    repo: Arc<SqliteGraphRepository>,
    config: TraversalConfig,
}

impl ShortestPathTraverser {
    pub fn new(repo: Arc<SqliteGraphRepository>, config: TraversalConfig) -> Self {
        Self { repo, config }
    }

    /// Finds the shortest path between source and target nodes using BFS
    pub async fn find_shortest_path(&self, project_id: &ares_core::ProjectId, source: &NodeId, target: &NodeId) -> Result<Vec<NodeId>, AresError> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent_map = std::collections::HashMap::new();

        queue.push_back((source.clone(), 0));
        visited.insert(source.clone());

        let mut found = false;

        while let Some((current, depth)) = queue.pop_front() {
            if current == *target {
                found = true;
                break;
            }

            if depth >= self.config.max_depth {
                continue;
            }

            let mut edges = self.repo.get_edges_from(&current)?;
            if let Ok(mut incoming) = self.repo.get_edges_to(&current) {
                edges.append(&mut incoming);
            }
            let mut neighbors_added = 0;

            for edge in edges {
                let neighbor_id = if edge.from_node_id.as_str() == current.as_str() {
                    NodeId::from(edge.to_node_id.as_str())
                } else {
                    NodeId::from(edge.from_node_id.as_str())
                };

                if !visited.contains(&neighbor_id) && neighbors_added < self.config.max_neighbors {
                    visited.insert(neighbor_id.clone());
                    parent_map.insert(neighbor_id.clone(), current.clone());
                    queue.push_back((neighbor_id, depth + 1));
                    neighbors_added += 1;
                }
            }
        }

        if !found {
            return Ok(Vec::new());
        }

        // Reconstruct path
        let mut path = Vec::new();
        let mut curr = target.clone();
        while curr != *source {
            path.push(curr.clone());
            if let Some(p) = parent_map.get(&curr) {
                curr = p.clone();
            } else {
                break;
            }
        }
        path.push(source.clone());
        path.reverse();

        Ok(path)
    }
}
