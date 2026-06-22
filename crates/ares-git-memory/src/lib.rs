pub mod models;
pub mod commits;
pub mod releases;
pub mod branches;
pub mod codeowners;
pub mod blame;

use std::path::Path;
use ares_core::ProjectId;
use models::GitMemoryResult;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct GitMemoryExtractor {
    project_path: std::path::PathBuf,
    depth: usize,
}

impl GitMemoryExtractor {
    pub fn new(path: &Path) -> Self {
        Self {
            project_path: path.to_path_buf(),
            depth: 500, // Default configurable depth
        }
    }

    pub fn set_depth(&mut self, depth: usize) {
        self.depth = depth;
    }

    pub fn extract(&self, project_id: &ProjectId) -> Result<GitMemoryResult, String> {
        let captured_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        
        let mut all_nodes = Vec::new();
        let mut all_edges = Vec::new();
        let mut sources = Vec::new();

        // 1. Commits
        if let Ok((nodes, edges)) = commits::CommitExtractor::extract(&self.project_path, project_id, self.depth, captured_at) {
            sources.push(models::MemorySource {
                name: "git_log".to_string(),
                tier: models::SourceTier::Repository,
                available: !nodes.is_empty(),
                captured: !nodes.is_empty(),
                node_count: nodes.len() as u64,
                edge_count: edges.len() as u64,
            });
            all_nodes.extend(nodes);
            all_edges.extend(edges);
        }

        // 2. Releases
        if let Ok((nodes, edges)) = releases::ReleaseExtractor::extract(&self.project_path, project_id, captured_at) {
            sources.push(models::MemorySource {
                name: "git_tag".to_string(),
                tier: models::SourceTier::Repository,
                available: !nodes.is_empty(),
                captured: !nodes.is_empty(),
                node_count: nodes.len() as u64,
                edge_count: edges.len() as u64,
            });
            all_nodes.extend(nodes);
            all_edges.extend(edges);
        }

        // 3. Branches
        if let Ok((nodes, edges)) = branches::BranchExtractor::extract(&self.project_path, project_id, captured_at) {
            sources.push(models::MemorySource {
                name: "git_branch".to_string(),
                tier: models::SourceTier::Repository,
                available: !nodes.is_empty(),
                captured: !nodes.is_empty(),
                node_count: nodes.len() as u64,
                edge_count: edges.len() as u64,
            });
            all_nodes.extend(nodes);
            all_edges.extend(edges);
        }

        // 4. CODEOWNERS
        if let Ok((nodes, edges)) = codeowners::CodeownersExtractor::extract(&self.project_path, project_id, captured_at) {
            sources.push(models::MemorySource {
                name: "codeowners".to_string(),
                tier: models::SourceTier::Explicit,
                available: !nodes.is_empty(),
                captured: !nodes.is_empty(),
                node_count: nodes.len() as u64,
                edge_count: edges.len() as u64,
            });
            all_nodes.extend(nodes);
            all_edges.extend(edges);
        }

        // 5. Blame
        if let Ok((nodes, edges)) = blame::BlameExtractor::extract(&self.project_path, project_id, captured_at) {
            sources.push(models::MemorySource {
                name: "git_blame".to_string(),
                tier: models::SourceTier::Repository,
                available: !nodes.is_empty(),
                captured: !nodes.is_empty(),
                node_count: nodes.len() as u64,
                edge_count: edges.len() as u64,
            });
            all_nodes.extend(nodes);
            all_edges.extend(edges);
        }

        Ok(GitMemoryResult {
            nodes: all_nodes,
            edges: all_edges,
            sources,
        })
    }
}
