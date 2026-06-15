use serde::{Deserialize, Serialize};
use ares_core::GraphNode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyTrace {
    pub target: String,
    pub path: Vec<GraphNode>,
    pub depth: usize,
}
