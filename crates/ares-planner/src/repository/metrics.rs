use ares_core::id::PlanId;
use ares_core::AresError;
use ares_store::db::Store;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetric {
    pub id: String,
    pub plan_id: PlanId,
    pub task_id: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub actual_duration: Option<f64>,
    pub worker_id: Option<String>,
    pub model_id: Option<String>,
    pub status: String,
    pub recorded_at: DateTime<Utc>,
}

pub struct SqliteMetricsRepository {
    #[allow(dead_code)]
    store: Arc<Store>,
}

impl SqliteMetricsRepository {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    pub fn get_by_plan(&self, _plan_id: &PlanId) -> Result<Vec<ExecutionMetric>, AresError> {
        Ok(vec![])
    }
}
