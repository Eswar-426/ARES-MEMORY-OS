use ares_core::types::node::GraphNode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskNode {
    pub node: GraphNode,
    pub description: String,
    pub severity: String,
}
