use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionType {
    Conversation,
    Execution,
    Planning,
    Collaboration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Paused,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceSession {
    pub session_id: String,
    pub session_type: SessionType,
    pub status: SessionStatus,
    pub context_id: Option<String>,
}
