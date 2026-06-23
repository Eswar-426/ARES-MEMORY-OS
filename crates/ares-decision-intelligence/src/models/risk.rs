use serde::{Deserialize, Serialize};
use ares_core::types::node::GraphNode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskNode {
    pub node: GraphNode,
    pub description: String,
    pub severity: String,
}
