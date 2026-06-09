use ares_core::id::{GoalId, PlanId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: PlanId,
    pub goal_id: GoalId,
    pub state: PlanState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlanState {
    Draft,
    Generated,
    Simulated,
    Approved,
    Scheduled,
    Executing,
    Completed,
    Failed,
    Replanned,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStateTransition {
    pub id: String,
    pub plan_id: PlanId,
    pub from_state: Option<PlanState>,
    pub to_state: PlanState,
    pub reason: Option<String>,
    pub transitioned_at: DateTime<Utc>,
}
