use ares_core::{AresError, NodeId, EdgeDirection, EdgeType};
use ares_retrieval::memory_retrieval_engine::MemoryRetrievalEngine;
use crate::models::{DecisionConflict, ConflictType};

pub struct DecisionConflictEngine<'a> {
    retrieval_engine: &'a MemoryRetrievalEngine,
}

impl<'a> DecisionConflictEngine<'a> {
    pub fn new(retrieval_engine: &'a MemoryRetrievalEngine) -> Self {
        Self { retrieval_engine }
    }

    pub fn detect_conflicts(&self, decision_id: &NodeId) -> Result<Vec<DecisionConflict>, AresError> {
        let mut conflicts = Vec::new();
        
        let contradictions = self.retrieval_engine.get_neighborhood(
            &decision_id.to_string(),
            EdgeDirection::Both,
            &[EdgeType::Contradicts],
        )?;

        for node in contradictions {
            conflicts.push(DecisionConflict {
                source_decision_id: decision_id.clone(),
                target_decision_id: node.id.clone(),
                conflict_type: ConflictType::ContradictoryDecision,
                rationale: "Decisions have a direct contradicts edge".into(),
            });
        }
        
        let superseded_by = self.retrieval_engine.get_neighborhood(
            &decision_id.to_string(),
            EdgeDirection::Incoming,
            &[EdgeType::Supersedes],
        )?;
        
        let node = self.retrieval_engine.get_node(&decision_id.to_string())?.unwrap();
        let is_active = node.properties.get("status").and_then(|v| v.as_str()) == Some("active");
        
        if !superseded_by.is_empty() && is_active {
            conflicts.push(DecisionConflict {
                source_decision_id: decision_id.clone(),
                target_decision_id: superseded_by[0].id.clone(),
                conflict_type: ConflictType::SupersededButActive,
                rationale: "Decision is active but superseded by another decision".into(),
            });
        }

        Ok(conflicts)
    }
}
