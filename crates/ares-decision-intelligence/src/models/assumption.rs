use ares_core::types::node::GraphNode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssumptionNode {
    pub node: GraphNode,
    pub description: String,
    pub is_valid: bool,
    pub is_stale: bool,
}
