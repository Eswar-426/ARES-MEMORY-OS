use crate::inference::registry::InferenceRegistry;
use crate::models::{EngineeringInsight, EngineeringQuery};
use crate::services::evidence_service::EvidenceService;
use ares_core::AresError;
use ares_store::Store;

// ═══════════════════════════════════════════════════════════════════
// IntelligenceFacade — the ONLY entry point for MCP
// ═══════════════════════════════════════════════════════════════════

/// Single entry point for all intelligence queries.
///
/// MCP handlers call `facade.execute(query)` and get back a
/// standardized `EngineeringInsight`. The facade owns the
/// `EvidenceService` (data) and `InferenceRegistry` (logic)
/// and orchestrates the pipeline:
///
/// ```text
/// EngineeringQuery
///     → EvidenceService.collect()
///         → EngineeringEvidence
///             → InferenceRegistry.dispatch()
///                 → EngineeringInsight
/// ```
pub struct IntelligenceFacade {
    evidence_service: EvidenceService,
    registry: InferenceRegistry,
}

impl IntelligenceFacade {
    pub fn new(store: Store) -> Self {
        Self {
            evidence_service: EvidenceService::new(store),
            registry: InferenceRegistry::new(),
        }
    }

    /// Execute an intelligence query and return a standardized insight.
    pub fn execute(&self, query: &EngineeringQuery) -> Result<EngineeringInsight, AresError> {
        // 1. Collect all available evidence
        let evidence = self.evidence_service.collect(query)?;

        // 2. Dispatch to the appropriate generator
        let insight = self
            .registry
            .dispatch(&evidence, &query.query_type)
            .ok_or_else(|| {
                AresError::Validation(format!(
                    "No generator registered for query type: {:?}",
                    query.query_type,
                ))
            })?;

        Ok(insight)
    }
}
