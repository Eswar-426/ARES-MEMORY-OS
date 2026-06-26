use crate::models::OwnershipMap;
use ares_core::{AresError, ProjectId};
use ares_retrieval::memory_retrieval_engine::MemoryRetrievalEngine;

#[allow(dead_code)]
pub struct OwnershipDiscoveryEngine<'a> {
    retrieval: &'a MemoryRetrievalEngine,
}

impl<'a> OwnershipDiscoveryEngine<'a> {
    pub fn new(retrieval: &'a MemoryRetrievalEngine) -> Self {
        Self { retrieval }
    }

    pub fn discover_ownership(&self, _project_id: &ProjectId) -> Result<OwnershipMap, AresError> {
        Ok(OwnershipMap { areas: vec![] })
    }
}
