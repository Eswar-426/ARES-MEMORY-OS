use serde::{Deserialize, Serialize};
use ares_core::types::node::GraphNode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionDNA {
    pub decision_node: GraphNode,
    pub reasoning_chain: String,
    pub assumptions: Vec<GraphNode>, // NodeType::Assumption
    pub alternatives: Vec<GraphNode>, // NodeType::Alternative
    pub risks: Vec<GraphNode>, // NodeType::Risk
    pub review_triggers: Vec<GraphNode>, // NodeType::ReviewTrigger
    pub impacted_artifacts: Vec<GraphNode>,
    pub supersedes: Vec<GraphNode>,
    pub superseded_by: Vec<GraphNode>,
}
