use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEvent {
    pub id: String,
    pub topic: String,
    pub payload: String, // JSON payload representing EventSchema
    pub created_at: String,
    pub published_at: Option<String>,
    pub status: String, // Pending, Published, Failed
    pub retry_count: i32,
}
