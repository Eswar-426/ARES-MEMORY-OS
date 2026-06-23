use ares_core::{AresError, NodeId, EdgeDirection, EdgeType};
use ares_retrieval::memory_retrieval_engine::MemoryRetrievalEngine;

pub struct DecisionReviewEngine<'a> {
    retrieval_engine: &'a MemoryRetrievalEngine,
}

impl<'a> DecisionReviewEngine<'a> {
    pub fn new(retrieval_engine: &'a MemoryRetrievalEngine) -> Self {
        Self { retrieval_engine }
    }

    pub fn requires_review(&self, decision_id: &NodeId) -> Result<bool, AresError> {
        let triggers = self.retrieval_engine.get_neighborhood(
            &decision_id.to_string(),
            EdgeDirection::Outgoing,
            &[EdgeType::HasReviewTrigger],
        )?;

        for trigger in triggers {
            if trigger.properties.get("is_triggered").and_then(|v| v.as_bool()).unwrap_or(false) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
