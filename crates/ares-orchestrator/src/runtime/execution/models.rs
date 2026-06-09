use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DistributedExecution {
    pub id: String,
    pub workflow_id: String,
    pub status: String, // Queued, Running, Completed, Failed
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DistributedExecutionAttempt {
    pub id: String,
    pub execution_id: String,
    pub worker_id: String,
    pub lease_id: String,
    pub attempt_number: i32,
    pub assigned_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub execution_duration_ms: Option<i64>,
    pub execution_node: String, // hostname
    pub status: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct WorkflowExecutionStep {
    pub id: String,
    pub attempt_id: String,
    pub step_name: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
}
