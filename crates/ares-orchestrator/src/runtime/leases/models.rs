use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct JobLease {
    pub id: String,
    pub worker_id: String,
    pub queue_id: String,
    pub workflow_id: String,
    pub execution_id: String,
    pub acquired_at: String,
    pub expires_at: String,
}
