use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub enum QueueStatus {
    Queued,
    Assigned,
    Running,
    Retrying,
    TimedOut,
    Orphaned,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct WorkflowQueueItem {
    pub id: String,
    pub workflow_id: String,
    pub priority: i32,
    pub status: QueueStatus,
    pub assigned_worker: Option<String>,
    pub retry_count: i32,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub execution_key: String,      // Idempotency key
    pub execution_checksum: String, // Idempotency checksum
}
