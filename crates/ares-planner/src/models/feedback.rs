use ares_core::id::{GoalId, PlanId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerFeedback {
    pub id: String,
    pub goal_id: GoalId,
    pub plan_id: PlanId,
    pub actual_duration: Option<f64>,
    pub actual_cost: Option<f64>,
    pub actual_success_rate: Option<f64>,
    pub agent_performance: Option<String>,
    pub recorded_at: DateTime<Utc>,
}
