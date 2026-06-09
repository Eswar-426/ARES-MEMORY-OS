use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EnqueueRequest {
    pub workflow_id: String,
    pub priority: i32,
    pub execution_key: String,
    pub execution_checksum: String,
}
