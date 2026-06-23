use crate::impact::{ImpactEngine, ImpactReport};
use crate::models::{KnowledgeNode, NodeType};
use crate::traversal::{MemoryTraversal, TraversalEngine};
use ares_core::AresError;
use std::sync::Arc;

pub struct CanonicalQueries {
    traversal: Arc<TraversalEngine>,
    impact: Arc<ImpactEngine>,
}

#[derive(Debug, Default)]
pub struct WhyResult {
    pub requirements: Vec<KnowledgeNode>,
    pub decisions: Vec<KnowledgeNode>,
    pub evidence: Vec<KnowledgeNode>,
}

#[derive(Debug, Default)]
pub struct OwnershipResult {
    pub owners: Vec<KnowledgeNode>,
    pub approvers: Vec<KnowledgeNode>,
    pub decisions: Vec<KnowledgeNode>,
}

#[derive(Debug, Default)]
pub struct DebtResult {
    pub gaps: Vec<KnowledgeNode>,
    pub root_causes: Vec<KnowledgeNode>,
    pub resolutions: Vec<KnowledgeNode>,
}

impl CanonicalQueries {
    pub fn new(traversal: Arc<TraversalEngine>, impact: Arc<ImpactEngine>) -> Self {
        Self { traversal, impact }
    }

    pub fn why_does_this_exist(&self, node_id: &str) -> Result<WhyResult, AresError> {
        let path = self.traversal.upstream(node_id, 10)?;
        let mut result = WhyResult::default();

        for node in path.nodes {
            match node.node_type {
                NodeType::Requirement => result.requirements.push(node),
                NodeType::Decision => result.decisions.push(node),
                NodeType::Evidence => result.evidence.push(node),
                _ => {}
            }
        }
        Ok(result)
    }

    pub fn who_owns_this(&self, node_id: &str) -> Result<OwnershipResult, AresError> {
        let up_path = self.traversal.upstream(node_id, 10)?;
        let down_path = self.traversal.downstream(node_id, 10)?;
        let mut result = OwnershipResult::default();
        let mut seen = std::collections::HashSet::new();

        let mut process = |nodes: Vec<KnowledgeNode>| {
            for node in nodes {
                if !seen.contains(&node.id) {
                    seen.insert(node.id.clone());
                    match node.node_type {
                        NodeType::Owner => {
                            result.owners.push(node.clone());
                            result.approvers.push(node);
                        }
                        NodeType::Decision => result.decisions.push(node),
                        _ => {}
                    }
                }
            }
        };

        process(up_path.nodes);
        process(down_path.nodes);

        Ok(result)
    }

    pub fn what_evidence_supports_this(
        &self,
        node_id: &str,
    ) -> Result<Vec<KnowledgeNode>, AresError> {
        let path = self.traversal.upstream(node_id, 10)?;
        let mut evidence = Vec::new();

        for node in path.nodes {
            if node.node_type == NodeType::Evidence {
                evidence.push(node);
            }
        }
        Ok(evidence)
    }

    pub fn what_knowledge_debt_exists(&self, node_id: &str) -> Result<DebtResult, AresError> {
        // Debt can be downstream (issues caused by this) or upstream (causes of this)
        // Usually down/around. We will search both for gaps.
        let up_path = self.traversal.upstream(node_id, 5)?;
        let down_path = self.traversal.downstream(node_id, 5)?;

        let mut result = DebtResult::default();
        let mut seen = std::collections::HashSet::new();

        let mut process_nodes = |nodes: Vec<KnowledgeNode>| {
            for node in nodes {
                if !seen.contains(&node.id) {
                    seen.insert(node.id.clone());
                    match node.node_type {
                        NodeType::Gap => result.gaps.push(node),
                        NodeType::RootCause => result.root_causes.push(node),
                        NodeType::Resolution => result.resolutions.push(node),
                        _ => {}
                    }
                }
            }
        };

        process_nodes(up_path.nodes);
        process_nodes(down_path.nodes);

        Ok(result)
    }

    pub fn what_breaks_if_changed(&self, node_id: &str) -> Result<ImpactReport, AresError> {
        self.impact.calculate_impact(node_id)
    }
}
