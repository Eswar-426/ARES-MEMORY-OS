use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCheckpoint {
    pub session_id: String,
    pub state_data: String,
    pub timestamp: u64,
}
