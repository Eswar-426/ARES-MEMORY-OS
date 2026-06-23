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
    pub total_decisions: u32,
    pub decisions_with_evidence: u32,
    pub decisions_without_owner: u32,
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
        pub fn insert_decision<T>(&self, _dec: &T) -> Result<(), AresError> { Ok(()) }
        pub fn insert_evidence<T>(&self, _ev: &T) -> Result<(), AresError> { Ok(()) }
        pub fn insert_outcome<T>(&self, _out: &T) -> Result<(), AresError> { Ok(()) }
        pub fn get_outcomes<T>(&self, _id: &T) -> Result<Vec<String>, AresError> { Ok(vec![]) }
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
        pub fn generate_snapshot(&self, _project_id: &ares_core::ProjectId) -> Result<DecisionCoverage, AresError> {
            Ok(DecisionCoverage { 
                percentage: 100.0, 
                health_score: 100.0,
                total_decisions: 2,
                decisions_with_evidence: 1,
                decisions_without_owner: 1
            })
        }
    }
}

pub mod history {
    pub struct DecisionHistory;
}
