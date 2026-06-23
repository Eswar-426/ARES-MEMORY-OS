use ares_core::{AresError, NodeId};
use ares_retrieval::memory_retrieval_engine::MemoryRetrievalEngine;
use crate::models::AssumptionNode;

pub struct AssumptionValidationEngine<'a> {
    retrieval_engine: &'a MemoryRetrievalEngine,
}

impl<'a> AssumptionValidationEngine<'a> {
    pub fn new(retrieval_engine: &'a MemoryRetrievalEngine) -> Self {
        Self { retrieval_engine }
    }

    pub fn validate_assumption(&self, assumption_id: &NodeId) -> Result<AssumptionNode, AresError> {
        let node = self.retrieval_engine.get_node(&assumption_id.to_string())?
            .ok_or_else(|| AresError::NotFound { resource_type: "Assumption".into(), id: assumption_id.to_string() })?;

        let is_valid = node.properties.get("is_valid").and_then(|v| v.as_bool()).unwrap_or(true);
        let is_stale = node.properties.get("is_stale").and_then(|v| v.as_bool()).unwrap_or(false);

        Ok(AssumptionNode {
            node: node.clone(),
            description: node.properties.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            is_valid,
            is_stale,
        })
    }
}
