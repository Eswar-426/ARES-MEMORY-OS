use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: Uuid,
    pub entity_type: String,
    pub name: String,
    pub description: Option<String>,
    pub properties: Value,
    pub embedding: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub confidence_score: f64,
    pub source_event_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityAlias {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub alias: String,
    pub normalized_alias: String,
    pub created_at: DateTime<Utc>,
}
