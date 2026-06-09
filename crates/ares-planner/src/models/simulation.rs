use ares_core::id::PlanId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanSimulationResult {
    pub plan_id: PlanId,
    pub success_probability: f64,
    pub expected_duration_seconds: f64,
    pub expected_cost: f64,
    pub risk_score: f64,
    pub simulated_at: DateTime<Utc>,
}
