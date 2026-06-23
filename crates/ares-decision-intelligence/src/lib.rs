pub mod engines;
pub mod models;

pub use models::DecisionStatus;

// Legacy shims for compatibility with other crates
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct DecisionSummary {
    pub id: String,
    pub title: String,
    pub approval_status: models::DecisionStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct DecisionCoverage {
    pub percentage: f32,
    pub health_score: f32,
}

pub mod integration {
    pub use crate::{DecisionCoverage, DecisionSummary};
}

pub mod storage {
    use ares_store::Store;
    use ares_core::AresError;
    pub struct DecisionStore;
    impl DecisionStore {
        pub fn new(_store: Store) -> Self { Self }
        pub fn get_evidence<T>(&self, _dec_id: &T) -> Result<Vec<String>, AresError> { Ok(vec![]) }
    }
    
    pub struct DecisionEdgeProvider;
    impl DecisionEdgeProvider {
        pub fn new(_store: Store) -> Self { Self }
        pub fn edges(&self) -> Result<Vec<ares_traceability::TraceabilityEdge>, AresError> { Ok(vec![]) }
    }
}

pub mod health {
    use ares_store::Store;
    use ares_core::{ProjectId, AresError};
    use crate::DecisionCoverage;
    pub struct DecisionHealthEngine;
    impl DecisionHealthEngine {
        pub fn new(_store: Store) -> Self { Self }
        pub fn generate_snapshot(&self, _project_id: &ProjectId) -> Result<DecisionCoverage, AresError> {
            Ok(DecisionCoverage { percentage: 100.0, health_score: 100.0 })
        }
    }
}
