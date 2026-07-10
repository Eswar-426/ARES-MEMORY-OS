use crate::models::{KnowledgeNode, NodeType};
use crate::traversal::{MemoryTraversal, TraversalEngine};
use ares_core::AresError;
use std::sync::Arc;

pub struct ImpactReport {
    pub total_score: u64,
    pub risk_level: String,
    pub impacted_nodes: Vec<KnowledgeNode>,
}

pub struct ImpactEngine {
    traversal: Arc<TraversalEngine>,
}

impl ImpactEngine {
    pub fn new(traversal: Arc<TraversalEngine>) -> Self {
        Self { traversal }
    }

    fn weight_for_node(node_type: &NodeType) -> u64 {
        match node_type {
            NodeType::Requirement => 10,
            NodeType::RequirementRevision => 10,
            NodeType::Decision => 8,
            NodeType::DecisionRevision => 8,
            NodeType::Evidence => 5,
            NodeType::RuntimeSignal => 4,
            NodeType::Gap => 8,
            NodeType::KnowledgeGap => 8,
            NodeType::RootCause => 9,
            NodeType::Architecture => 7,
            NodeType::Resolution => 2,
            NodeType::Outcome => 6,
            NodeType::CodeArtifact => 5,
            NodeType::Test => 4,
            NodeType::Owner => 3,
            NodeType::Repository => 1,
            NodeType::Project => 1,
            NodeType::RepositoryEvent => 1,
            NodeType::RepositorySnapshot => 1,
            _ => 1,
        }
    }

    pub fn calculate_impact(&self, start_node_id: &str) -> Result<ImpactReport, AresError> {
        // Downstream traversal to find all impacted nodes
        let path = self.traversal.downstream(start_node_id, 10)?;

        let mut total_score = 0;
        let mut impacted_nodes = Vec::new();

        for node in path.nodes {
            // We don't want to count the starting node itself in the impact risk score,
            // but for simplicity we will include it as part of the impacted chain if it's the root.
            if node.id != start_node_id {
                total_score += Self::weight_for_node(&node.node_type);
            }
            impacted_nodes.push(node);
        }

        let risk_level = if total_score >= 50 {
            "CRITICAL".to_string()
        } else if total_score >= 25 {
            "HIGH".to_string()
        } else if total_score >= 10 {
            "MEDIUM".to_string()
        } else {
            "LOW".to_string()
        };

        Ok(ImpactReport {
            total_score,
            risk_level,
            impacted_nodes,
        })
    }
}
