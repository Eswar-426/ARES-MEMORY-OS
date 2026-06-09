use crate::models::candidate::PlanCandidate;
use crate::models::goal::Goal;
use ares_core::id::{GoalId, PlanId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalCreatedPayload {
    pub goal: Goal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalDecomposedPayload {
    pub parent_goal_id: GoalId,
    pub child_goals: Vec<Goal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanGeneratedPayload {
    pub candidates: Vec<PlanCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanApprovedPayload {
    pub plan_id: PlanId,
    pub approved_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStartedPayload {
    pub plan_id: PlanId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanCompletedPayload {
    pub plan_id: PlanId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplanningTriggeredPayload {
    pub plan_id: PlanId,
    pub reason: String,
}
