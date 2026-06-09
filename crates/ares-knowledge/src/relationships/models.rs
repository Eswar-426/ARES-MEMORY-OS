use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: Uuid,
    pub source_entity: Uuid,
    pub target_entity: Uuid,
    pub relationship_type: String,
    pub properties: Value,
    pub embedding: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub confidence_score: f64,
    pub evidence_count: i64,
    pub source_event_id: Option<Uuid>,
}
