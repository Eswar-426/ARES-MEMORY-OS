use ares_core::{AresError, NodeId, GraphNode};
use ares_store::repositories::graph::SqliteGraphRepository;
use std::sync::Arc;
use std::collections::HashSet;

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
    pub async fn resolve_files(&self, project_id: &ares_core::ProjectId, file_paths: &[String]) -> Result<Vec<GraphNode>, AresError> {
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
            let pagination = ares_core::types::pagination::Pagination { page: 1, page_size: 50 };
            if let Ok(page) = self.repo.list_nodes_paginated(project_id, None, Some(path), &pagination) {
                for node in page.items {
                    if seen_ids.insert(node.id.clone()) {
                        // Distinguish definition nodes (structs, functions, files with file_path)
                        // from unresolved import nodes (modules with no file_path and "unresolved" property)
                        let is_unresolved = node.properties.get("unresolved")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        
                        if is_unresolved {
                            // Skip unresolved import references — they add noise
                            continue;
                        }
                        
                        let is_definition = matches!(node.node_type,
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
}
