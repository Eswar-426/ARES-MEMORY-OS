use crate::models::DependencyMap;
use ares_core::{AresError, ProjectId};
use ares_retrieval::memory_retrieval_engine::MemoryRetrievalEngine;
use std::collections::HashMap;

#[allow(dead_code)]
pub struct DependencyDiscoveryEngine<'a> {
    retrieval: &'a MemoryRetrievalEngine,
}

impl<'a> DependencyDiscoveryEngine<'a> {
    pub fn new(retrieval: &'a MemoryRetrievalEngine) -> Self {
        Self { retrieval }
    }

    pub fn discover_dependencies(
        &self,
        _project_id: &ProjectId,
    ) -> Result<DependencyMap, AresError> {
        Ok(DependencyMap {
            external_dependencies: vec![],
            internal_dependencies: HashMap::new(),
        })
    }
}
