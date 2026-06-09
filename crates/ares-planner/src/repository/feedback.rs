use crate::models::feedback::PlannerFeedback;
use ares_core::id::GoalId;
use ares_core::AresError;
use ares_store::db::Store;
use std::sync::Arc;

pub struct SqliteFeedbackRepository {
    #[allow(dead_code)]
    store: Arc<Store>,
}

impl SqliteFeedbackRepository {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    pub fn get_by_goal(&self, _goal_id: &GoalId) -> Result<Vec<PlannerFeedback>, AresError> {
        Ok(vec![])
    }
}
