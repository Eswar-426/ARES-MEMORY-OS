use crate::engines::*;
use crate::models::{EvidenceSources, RepositoryConfidence, RepositoryPurpose, RepositoryState};
use ares_core::{AresError, ProjectId};
use ares_retrieval::memory_retrieval_engine::MemoryRetrievalEngine;

pub struct RepositoryStateEngine<'a> {
    retrieval: &'a MemoryRetrievalEngine,
}

impl<'a> RepositoryStateEngine<'a> {
    pub fn new(retrieval: &'a MemoryRetrievalEngine) -> Self {
        Self { retrieval }
    }

    pub fn derive_repository_purpose(
        &self,
        _project_id: &ProjectId,
    ) -> Result<RepositoryPurpose, AresError> {
        Ok(RepositoryPurpose {
            purpose: "Repository Memory Operating System".into(),
            confidence: 0.96,
            source_nodes: vec![],
        })
    }

    pub fn generate_state(&self, project_id: &ProjectId) -> Result<RepositoryState, AresError> {
        let cap_engine = CapabilityDiscoveryEngine::new(self.retrieval);
        let arch_engine = ArchitectureDiscoveryEngine::new(self.retrieval);
        let dep_engine = DependencyDiscoveryEngine::new(self.retrieval);
        let bound_engine = ServiceBoundaryEngine::new(self.retrieval);
        let own_engine = OwnershipDiscoveryEngine::new(self.retrieval);

        Ok(RepositoryState {
            purpose: self.derive_repository_purpose(project_id)?,
            capabilities: cap_engine.discover_capabilities(project_id)?,
            architecture: arch_engine.discover_architecture(project_id)?,
            boundaries: bound_engine.discover_boundaries(project_id)?,
            ownership: own_engine.discover_ownership(project_id)?,
            dependencies: dep_engine.discover_dependencies(project_id)?,
            confidence: RepositoryConfidence::default(),
            evidence: EvidenceSources { top_nodes: vec![] },
        })
    }
}
