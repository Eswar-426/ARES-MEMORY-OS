use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalState {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub status: String,
    pub progress: f64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub goal_id: Uuid,
    pub steps: Vec<Uuid>,
    pub expected_capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainPlanResponse {
    pub execution_plan: ExecutionPlan,
    pub rationale: String,
    pub dependencies_satisfied: bool,
    pub conflicting_goals: Vec<Uuid>,
}
