use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DeadLetterItem {
    pub id: String,
    pub original_queue_id: String,
    pub workflow_id: String,
    pub execution_key: String,
    pub failure_reason: String,
    pub failed_at: String,
    pub attempt_count: i32,
}
