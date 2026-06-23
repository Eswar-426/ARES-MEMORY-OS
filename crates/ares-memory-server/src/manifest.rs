use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildManifest {
    pub version: String,
    pub last_build: String,
    pub stages_completed: Vec<String>,
    pub memory_graph_hash: String,
    pub repository_hash: String,
}
