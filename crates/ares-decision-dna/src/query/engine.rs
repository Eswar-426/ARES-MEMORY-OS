use crate::models::DecisionId;
use ares_core::types::node::{EdgeType, NodeType};

pub struct DecisionQueryEngine;

pub struct TraversalQuery {
    pub start_node: DecisionId,
    pub steps: Vec<TraversalStep>,
}

pub struct TraversalStep {
    pub edge_type: EdgeType,
    pub target_node_type: NodeType,
}

impl DecisionQueryEngine {
    pub fn why_was_this_made(decision_id: DecisionId) -> TraversalQuery {
        TraversalQuery {
            start_node: decision_id,
            steps: vec![TraversalStep {
                edge_type: EdgeType::MotivatedBy,
                target_node_type: NodeType::Requirement, // Or Feature, Bug
            }],
        }
    }

    pub fn which_files_are_affected(decision_id: DecisionId) -> TraversalQuery {
        TraversalQuery {
            start_node: decision_id,
            steps: vec![TraversalStep {
                edge_type: EdgeType::Impacts,
                target_node_type: NodeType::File,
            }],
        }
    }

    pub fn what_superseded_this(decision_id: DecisionId) -> TraversalQuery {
        TraversalQuery {
            start_node: decision_id,
            steps: vec![TraversalStep {
                edge_type: EdgeType::Supersedes,
                target_node_type: NodeType::Decision, // Reverse traversal usually needed depending on graph implementation
            }],
        }
    }
}
