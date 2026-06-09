use ares_core::id::PlanId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEstimate {
    pub plan_id: PlanId,
    pub cpu_hours: Option<f64>,
    pub memory_mb: Option<f64>,
    pub agent_count: Option<u32>,
    pub token_budget: Option<u64>,
    pub estimated_cost: Option<f64>,
    pub estimated_at: DateTime<Utc>,
}
