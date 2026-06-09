use crate::models::candidate::PlanCandidate;
use ares_core::id::PlanCandidateId;
use ares_core::AresError;
use ares_store::db::Store;
use std::sync::Arc;

pub struct SqliteCandidateRepository {
    #[allow(dead_code)]
    store: Arc<Store>,
}

impl SqliteCandidateRepository {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    pub fn get(&self, _id: &PlanCandidateId) -> Result<Option<PlanCandidate>, AresError> {
        Ok(None)
    }
}
