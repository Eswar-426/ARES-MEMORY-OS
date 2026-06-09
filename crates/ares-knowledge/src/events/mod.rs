use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeEventType {
    EntityCreated,
    EntityUpdated,
    EntityDeleted,
    RelationshipCreated,
    RelationshipDeleted,
    EntityMerged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEvent {
    pub id: Uuid,
    pub event_type: KnowledgeEventType,
    pub payload: serde_json::Value,
    pub processed_at: DateTime<Utc>,
    pub status: String,
}

impl KnowledgeEvent {
    pub fn new(event_type: KnowledgeEventType, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::now_v7(),
            event_type,
            payload,
            processed_at: Utc::now(),
            status: "PENDING".to_string(),
        }
    }
}
