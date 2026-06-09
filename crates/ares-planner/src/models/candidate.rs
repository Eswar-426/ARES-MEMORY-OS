use ares_core::id::{GoalId, PlanCandidateId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanCandidate {
    pub id: PlanCandidateId,
    pub goal_id: GoalId,
    pub dag_json: String,
    pub score: f64,
    pub estimated_cost: Option<f64>,
    pub estimated_duration: Option<f64>,
    pub generated_at: DateTime<Utc>,
}

impl PlanCandidate {
    #[cfg(test)]
    pub fn new_test(_plan_id: ares_core::id::PlanId, dag_json: String) -> Self {
        Self {
            id: ares_core::id::PlanCandidateId::new(),
            goal_id: ares_core::id::GoalId::new(),
            dag_json,
            score: 0.0,
            estimated_cost: None,
            estimated_duration: None,
            generated_at: chrono::Utc::now(),
        }
    }
}
