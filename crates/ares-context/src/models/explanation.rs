use ares_core::GraphNode;
use serde::{Deserialize, Serialize};

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
