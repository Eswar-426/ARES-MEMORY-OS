use ares_core::GraphNode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyTrace {
    pub target: String,
    pub path: Vec<GraphNode>,
    pub depth: usize,
}
