use ares_core::{AresError, NodeId, GraphNode};
use ares_store::repositories::graph::SqliteGraphRepository;
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

    /// Resolves file nodes by file paths
    pub async fn resolve_files(&self, project_id: &ares_core::ProjectId, file_paths: &[String]) -> Result<Vec<GraphNode>, AresError> {
        let mut nodes = Vec::new();
        for path in file_paths {
            let matches = self.repo.get_by_file_path(project_id, path)?;
            nodes.extend(matches);
        }
        Ok(nodes)
    }
}
