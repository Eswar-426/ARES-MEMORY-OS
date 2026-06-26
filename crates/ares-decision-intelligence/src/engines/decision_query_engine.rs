use crate::models::DecisionDNA;
use ares_core::{types::node::NodeType, AresError, EdgeDirection, EdgeType, NodeId, ProjectId};
use ares_retrieval::memory_retrieval_engine::MemoryRetrievalEngine;

pub struct DecisionQueryEngine<'a> {
    retrieval_engine: &'a MemoryRetrievalEngine,
}

impl<'a> DecisionQueryEngine<'a> {
    pub fn new(retrieval_engine: &'a MemoryRetrievalEngine) -> Self {
        Self { retrieval_engine }
    }

    pub fn get_decisions_for_file(
        &self,
        project_id: &ProjectId,
        file_id: &NodeId,
    ) -> Result<Vec<DecisionDNA>, AresError> {
        let upstream = self.retrieval_engine.get_neighborhood(
            file_id.as_ref(),
            EdgeDirection::Incoming,
            &[EdgeType::Drives, EdgeType::Implements],
        )?;

        let mut decisions = Vec::new();
        for node in upstream {
            if node.node_type == NodeType::Decision {
                decisions.push(self.get_decision_dna(project_id, &node.id)?);
            } else if node.node_type == NodeType::Architecture {
                let arch_upstream = self.retrieval_engine.get_neighborhood(
                    node.id.as_ref(),
                    EdgeDirection::Incoming,
                    &[EdgeType::Drives],
                )?;
                for arch_node in arch_upstream {
                    if arch_node.node_type == NodeType::Decision {
                        decisions.push(self.get_decision_dna(project_id, &arch_node.id)?);
                    }
                }
            }
        }

        Ok(decisions)
    }

    pub fn get_decisions_for_feature(
        &self,
        project_id: &ProjectId,
        feature_id: &NodeId,
    ) -> Result<Vec<DecisionDNA>, AresError> {
        let upstream = self.retrieval_engine.get_neighborhood(
            feature_id.as_ref(),
            EdgeDirection::Incoming,
            &[EdgeType::MotivatedBy, EdgeType::Drives],
        )?;

        let mut decisions = Vec::new();
        for node in upstream {
            if node.node_type == NodeType::Decision {
                decisions.push(self.get_decision_dna(project_id, &node.id)?);
            }
        }
        Ok(decisions)
    }

    pub fn get_decisions_for_architecture(
        &self,
        project_id: &ProjectId,
        arch_id: &NodeId,
    ) -> Result<Vec<DecisionDNA>, AresError> {
        let upstream = self.retrieval_engine.get_neighborhood(
            arch_id.as_ref(),
            EdgeDirection::Incoming,
            &[EdgeType::Drives],
        )?;

        let mut decisions = Vec::new();
        for node in upstream {
            if node.node_type == NodeType::Decision {
                decisions.push(self.get_decision_dna(project_id, &node.id)?);
            }
        }
        Ok(decisions)
    }

    pub fn get_related_decisions(
        &self,
        project_id: &ProjectId,
        decision_id: &NodeId,
    ) -> Result<Vec<DecisionDNA>, AresError> {
        let related = self.retrieval_engine.get_neighborhood(
            decision_id.as_ref(),
            EdgeDirection::Both,
            &[EdgeType::RelatedTo],
        )?;

        let mut decisions = Vec::new();
        for node in related {
            if node.node_type == NodeType::Decision {
                decisions.push(self.get_decision_dna(project_id, &node.id)?);
            }
        }
        Ok(decisions)
    }

    pub fn get_decision_dna(
        &self,
        _project_id: &ProjectId,
        decision_id: &NodeId,
    ) -> Result<DecisionDNA, AresError> {
        let decision_node = self
            .retrieval_engine
            .get_node(decision_id.as_ref())?
            .ok_or_else(|| AresError::NotFound {
                resource_type: "Decision",
                id: decision_id.to_string(),
            })?;

        let reasoning_chain = decision_node
            .properties
            .get("reasoning_chain")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let assumptions = self.retrieval_engine.get_neighborhood(
            decision_id.as_ref(),
            EdgeDirection::Outgoing,
            &[EdgeType::HasAssumption],
        )?;
        let alternatives = self.retrieval_engine.get_neighborhood(
            decision_id.as_ref(),
            EdgeDirection::Outgoing,
            &[EdgeType::HasAlternative],
        )?;
        let risks = self.retrieval_engine.get_neighborhood(
            decision_id.as_ref(),
            EdgeDirection::Outgoing,
            &[EdgeType::HasRisk],
        )?;
        let review_triggers = self.retrieval_engine.get_neighborhood(
            decision_id.as_ref(),
            EdgeDirection::Outgoing,
            &[EdgeType::HasReviewTrigger],
        )?;

        let impacted_artifacts = self.retrieval_engine.get_neighborhood(
            decision_id.as_ref(),
            EdgeDirection::Outgoing,
            &[EdgeType::Impacts, EdgeType::Drives],
        )?;

        let supersedes = self.retrieval_engine.get_neighborhood(
            decision_id.as_ref(),
            EdgeDirection::Outgoing,
            &[EdgeType::Supersedes],
        )?;
        let superseded_by = self.retrieval_engine.get_neighborhood(
            decision_id.as_ref(),
            EdgeDirection::Incoming,
            &[EdgeType::Supersedes],
        )?;

        Ok(DecisionDNA {
            decision_node,
            reasoning_chain,
            assumptions,
            alternatives,
            risks,
            review_triggers,
            impacted_artifacts,
            supersedes,
            superseded_by,
        })
    }
}
