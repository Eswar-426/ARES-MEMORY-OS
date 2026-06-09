use crate::models::approval::PlanApproval;
use ares_core::id::PlanId;
use ares_core::AresError;
use ares_store::db::Store;
use std::sync::Arc;

pub struct SqliteApprovalRepository {
    #[allow(dead_code)]
    store: Arc<Store>,
}

impl SqliteApprovalRepository {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    pub fn create(&self, _approval: &PlanApproval) -> Result<(), AresError> {
        Ok(())
    }

    pub fn get_by_plan(&self, _plan_id: &PlanId) -> Result<Option<PlanApproval>, AresError> {
        Ok(None)
    }
}
