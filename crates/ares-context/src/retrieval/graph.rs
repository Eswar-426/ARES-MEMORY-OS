use ares_core::{AresError, GraphNode, NodeId};
use ares_store::repositories::graph::SqliteGraphRepository;
use std::collections::HashSet;
use std::sync::Arc;

pub struct GraphRetriever {
    repo: Arc<SqliteGraphRepository>,
}

impl GraphRetriever {
    pub fn new(repo: Arc<SqliteGraphRepository>) -> Self {
        Self { repo }
    }

    /// Fetches target nodes by explicit IDs
    pub async fn fetch_nodes(&self, node_ids: &[NodeId]) -> Result<Vec<GraphNode>, AresError> {
        let mut nodes = Vec::new();
        for id in node_ids {
            if let Some(node) = self.repo.get_node(id)? {
                nodes.push(node);
            }
        }
        Ok(nodes)
    }

    /// Resolves nodes by file paths or symbols.
    /// Returns deduplicated nodes, prioritizing definition nodes over import references.
    pub async fn resolve_files(
        &self,
        project_id: &ares_core::ProjectId,
        file_paths: &[String],
    ) -> Result<Vec<GraphNode>, AresError> {
        let mut seen_ids = HashSet::new();
        let mut definition_nodes = Vec::new();
        let mut other_nodes = Vec::new();

        for path in file_paths {
            // 1. Try exact file_path match first
            let matches = self.repo.get_by_file_path(project_id, path)?;
            for node in matches {
                if seen_ids.insert(node.id.clone()) {
                    definition_nodes.push(node);
                }
            }

            // 2. Search by name/label
            let pagination = ares_core::types::pagination::Pagination {
                page: 1,
                page_size: 50,
            };
            if let Ok(page) =
                self.repo
                    .list_nodes_paginated(project_id, None, Some(path), &pagination)
            {
                for node in page.items {
                    if seen_ids.insert(node.id.clone()) {
                        // Distinguish definition nodes (structs, functions, files with file_path)
                        // from unresolved import nodes (modules with no file_path and "unresolved" property)
                        let is_unresolved = node
                            .properties
                            .get("unresolved")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        if is_unresolved {
                            // Skip unresolved import references — they add noise
                            continue;
                        }

                        let is_definition = matches!(
                            node.node_type,
                            ares_core::NodeType::Struct
                                | ares_core::NodeType::Function
                                | ares_core::NodeType::Enum
                                | ares_core::NodeType::Trait
                                | ares_core::NodeType::Class
                                | ares_core::NodeType::File
                        );

                        if is_definition {
                            definition_nodes.push(node);
                        } else {
                            other_nodes.push(node);
                        }
                    }
                }
            }
        }

        // Return definition nodes first, then other nodes
        definition_nodes.extend(other_nodes);
        Ok(definition_nodes)
    }

    /// Computes the number of unique nodes reachable from the seeds within a given depth
    pub async fn count_reachable_nodes(
        &self,
        seed_ids: &[NodeId],
        depth: usize,
    ) -> Result<usize, AresError> {
        if depth == 0 || seed_ids.is_empty() {
            return Ok(seed_ids.len());
        }

        let mut current_layer = seed_ids
            .iter()
            .map(|id| id.as_str().to_string())
            .collect::<Vec<_>>();
        let mut visited = std::collections::HashSet::new();
        for id in &current_layer {
            visited.insert(id.clone());
        }

        for _ in 0..depth {
            if current_layer.is_empty() {
                break;
            }

            // Fetch neighbors for current layer (all directions)
            // Note: Since this is telemetry, we can do a simple batched query
            let mut next_layer = Vec::new();
            for chunk in current_layer.chunks(50) {
                let in_clause = chunk
                    .iter()
                    .map(|id| format!("'{}'", id))
                    .collect::<Vec<_>>()
                    .join(",");
                let query = format!(
                    "SELECT from_node_id, to_node_id FROM graph_edges WHERE (from_node_id IN ({}) OR to_node_id IN ({})) AND valid_until IS NULL",
                    in_clause, in_clause
                );

                let conn = self.repo.store().get_conn()?;
                let mut stmt = conn
                    .prepare(&query)
                    .map_err(|e| AresError::db(e.to_string()))?;
                let mut rows = stmt.query(()).map_err(|e| AresError::db(e.to_string()))?;
                while let Some(row) = rows.next().map_err(|e| AresError::db(e.to_string()))? {
                    let from: String = row.get(0).unwrap();
                    let to: String = row.get(1).unwrap();
                    if !visited.contains(&from) {
                        visited.insert(from.clone());
                        next_layer.push(from);
                    }
                    if !visited.contains(&to) {
                        visited.insert(to.clone());
                        next_layer.push(to);
                    }
                }
            }
            current_layer = next_layer;
        }

        Ok(visited.len())
    }
}
