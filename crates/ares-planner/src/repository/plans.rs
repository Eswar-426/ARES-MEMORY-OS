use crate::models::plan::Plan;
use ares_core::id::PlanId;
use ares_core::AresError;
use ares_store::db::Store;
use std::sync::Arc;

pub struct SqlitePlanRepository {
    #[allow(dead_code)]
    store: Arc<Store>,
}

impl SqlitePlanRepository {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    pub fn get(&self, _id: &PlanId) -> Result<Option<Plan>, AresError> {
        Ok(None)
    }
}
