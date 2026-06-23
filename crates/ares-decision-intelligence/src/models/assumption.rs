use serde::{Deserialize, Serialize};
use ares_core::types::node::GraphNode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssumptionNode {
    pub node: GraphNode,
    pub description: String,
    pub is_valid: bool,
    pub is_stale: bool,
}
