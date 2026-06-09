use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceRecord {
    pub id: Uuid,
    pub entity_id: Option<Uuid>,
    pub relationship_id: Option<Uuid>,
    pub event_id: Uuid,
    pub source_type: String,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
}
