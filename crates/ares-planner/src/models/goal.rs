use ares_core::id::GoalId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: GoalId,
    pub title: String,
    pub description: Option<String>,
    pub priority: GoalPriority,
    pub deadline: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GoalPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GoalState {
    Draft,
    Ready,
    Planning,
    Planned,
    Executing,
    Completed,
    PlanningFailed,
    ExecutionFailed,
    Cancelled,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalStateRecord {
    pub goal_id: GoalId,
    pub state: GoalState,
    pub confidence: Option<f64>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalStateTransition {
    pub id: String,
    pub goal_id: GoalId,
    pub from_state: Option<GoalState>,
    pub to_state: GoalState,
    pub reason: Option<String>,
    pub transitioned_at: DateTime<Utc>,
}
