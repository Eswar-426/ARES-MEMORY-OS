use serde::{Deserialize, Serialize};
use ares_core::GraphNode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileExplanation {
    pub file_path: String,
    pub definitions: Vec<GraphNode>,
    pub dependencies: Vec<String>,
    pub related_components: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInsight {
    pub summary: String,
}
