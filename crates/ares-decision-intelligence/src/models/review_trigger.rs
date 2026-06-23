use serde::{Deserialize, Serialize};
use ares_core::types::node::GraphNode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewTriggerNode {
    pub node: GraphNode,
    pub condition: String,
    pub is_triggered: bool,
}
