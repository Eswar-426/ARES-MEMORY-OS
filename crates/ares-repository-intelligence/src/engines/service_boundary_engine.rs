use crate::models::ServiceBoundaries;
use ares_core::{AresError, ProjectId};
use ares_retrieval::memory_retrieval_engine::MemoryRetrievalEngine;

pub struct ServiceBoundaryEngine<'a> {
    retrieval: &'a MemoryRetrievalEngine,
}

impl<'a> ServiceBoundaryEngine<'a> {
    pub fn new(retrieval: &'a MemoryRetrievalEngine) -> Self {
        Self { retrieval }
    }

    pub fn discover_boundaries(
        &self,
        _project_id: &ProjectId,
    ) -> Result<ServiceBoundaries, AresError> {
        Ok(ServiceBoundaries { boundaries: vec![] })
    }
}
