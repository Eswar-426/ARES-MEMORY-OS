use crate::models::{Capability, CapabilityMap};
use ares_core::{AresError, NodeType, ProjectId};
use ares_retrieval::memory_retrieval_engine::MemoryRetrievalEngine;

pub struct CapabilityDiscoveryEngine<'a> {
    retrieval: &'a MemoryRetrievalEngine,
}

impl<'a> CapabilityDiscoveryEngine<'a> {
    pub fn new(retrieval: &'a MemoryRetrievalEngine) -> Self {
        Self { retrieval }
    }

    pub fn discover_capabilities(
        &self,
        project_id: &ProjectId,
    ) -> Result<CapabilityMap, AresError> {
        let reqs = self.retrieval.find_by_type(project_id, NodeType::Requirement)?;
        let mut caps = Vec::new();

        for req in reqs.into_iter() {
            caps.push(Capability {
                name: req.label.clone(),
                description: req.label.clone(),
                requirement_nodes: vec![req.id.clone()],
                decision_nodes: vec![],
                architecture_nodes: vec![],
                code_nodes: vec![],
            });
        }

        Ok(CapabilityMap { capabilities: caps })
    }
}
