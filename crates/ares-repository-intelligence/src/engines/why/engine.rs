use crate::core::capabilities::Capability;
use crate::core::context::RepositoryContext;
use crate::core::engine::{
    EngineDescriptor, EngineExecutionResult, EngineId, EngineInput, RepositoryEngine,
};
use crate::core::errors::{EngineError, EngineResult};
use crate::core::evidence::EvidenceBundle;
use crate::core::metadata::ExecutionMetadata;
use ares_knowledge_graph::impact::ImpactEngine;
use ares_knowledge_graph::queries::CanonicalQueries;
use ares_knowledge_graph::store::KnowledgeGraphStore;
use ares_knowledge_graph::traversal::TraversalEngine;
use ares_store::Store;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

pub struct RepositoryWhyEngine {
    store: Store,
}

impl RepositoryWhyEngine {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

#[async_trait]
impl RepositoryEngine for RepositoryWhyEngine {
    fn descriptor(&self) -> EngineDescriptor {
        EngineDescriptor {
            id: EngineId::WhyExists,
            version: "0.1.0".to_string(),
            capabilities: vec![
                Capability::WhyExists,
                Capability::GraphSearch,
                Capability::GitHistory,
            ],
            planner_api_version: 1,
        }
    }

    async fn execute(
        &self,
        _context: &RepositoryContext,
        input: EngineInput,
    ) -> EngineResult<EngineExecutionResult> {
        let start = std::time::Instant::now();

        let entity_id = match input {
            EngineInput::NodeId(id) => id,
            _ => {
                return Err(EngineError::ExecutionError(
                    "WhyExists requires a NodeId input".to_string(),
                ))
            }
        };

        let kg_store = KnowledgeGraphStore::new(Arc::new(self.store.clone()));
        let traversal = Arc::new(TraversalEngine::new(Arc::new(kg_store)));
        let impact = Arc::new(ImpactEngine::new(traversal.clone()));
        let queries = CanonicalQueries::new(traversal, impact);

        let why_result = queries
            .why_does_this_exist(&entity_id)
            .map_err(|e| EngineError::ExecutionError(format!("WhyExists failed: {}", e)))?;

        let mut bundle = EvidenceBundle::default();

        bundle.architecture = Some(crate::core::evidence::ArchitectureEvidence {
            decisions: why_result
                .decisions
                .iter()
                .map(|n| n.id.to_string())
                .collect(),
            requirements: why_result
                .requirements
                .iter()
                .map(|n| n.id.to_string())
                .collect(),
            adrs: vec![],
        });

        Ok(EngineExecutionResult {
            descriptor: self.descriptor(),
            engine_id: EngineId::WhyExists,
            capability: Capability::WhyExists,
            evidence: bundle,
            metadata: ExecutionMetadata {
                engine: "WhyExists".to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
                cache_hit: false,
                confidence: 1.0,
                errors: vec![],
                warnings: vec![],
                retry_count: 0,
                sources_used: vec!["graph".to_string()],
            },
            diagnostics: HashMap::new(),
            warnings: vec![],
            errors: vec![],
            artifacts: vec![],
        })
    }
}
