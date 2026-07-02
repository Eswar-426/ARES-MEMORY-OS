use crate::core::capabilities::Capability;
use crate::core::context::RepositoryContext;
use crate::core::engine::{
    EngineDescriptor, EngineExecutionResult, EngineId, EngineInput, RepositoryEngine,
};
use crate::core::errors::{EngineError, EngineResult};
use crate::core::evidence::{EvidenceBundle, RuntimeEvidence};
use crate::core::metadata::ExecutionMetadata;
use ares_knowledge_graph::impact::ImpactEngine;
use ares_knowledge_graph::queries::CanonicalQueries;
use ares_knowledge_graph::store::KnowledgeGraphStore;
use ares_knowledge_graph::traversal::TraversalEngine;
use ares_store::Store;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

pub struct RepositoryImpactEngine {
    store: Store,
}

impl RepositoryImpactEngine {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

#[async_trait]
impl RepositoryEngine for RepositoryImpactEngine {
    fn descriptor(&self) -> EngineDescriptor {
        EngineDescriptor {
            id: EngineId::Impact,
            version: "0.1.0".to_string(),
            capabilities: vec![Capability::ImpactAnalysis, Capability::GraphSearch],
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
                    "Impact requires a NodeId input".to_string(),
                ))
            }
        };

        let kg_store = KnowledgeGraphStore::new(Arc::new(self.store.clone()));
        let traversal = Arc::new(TraversalEngine::new(Arc::new(kg_store)));
        let impact = Arc::new(ImpactEngine::new(traversal.clone()));
        let queries = CanonicalQueries::new(traversal, impact);

        let impact_result = queries
            .what_breaks_if_changed(&entity_id)
            .map_err(|e| EngineError::ExecutionError(format!("Impact analysis failed: {}", e)))?;

        let mut bundle = EvidenceBundle::default();

        let mut stats = HashMap::new();
        stats.insert("total_score".to_string(), impact_result.total_score as f64);

        let mut metadata = HashMap::new();
        metadata.insert("risk_level".to_string(), impact_result.risk_level);

        bundle.runtime = Some(RuntimeEvidence {
            confidence: 1.0,
            statistics: stats,
            sources: impact_result
                .impacted_nodes
                .iter()
                .map(|n| n.id.to_string())
                .collect(),
        });
        bundle.metadata = metadata;

        Ok(EngineExecutionResult {
            descriptor: self.descriptor(),
            engine_id: EngineId::Impact,
            capability: Capability::ImpactAnalysis,
            evidence: bundle,
            metadata: ExecutionMetadata {
                engine: "Impact".to_string(),
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
