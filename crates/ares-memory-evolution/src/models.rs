use serde::{Deserialize, Serialize};

#[derive(utoipa::ToSchema, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeType {
    Created,
    Updated,
    Superseded,
    Approved,
    Rejected,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRevision {
    pub revision_id: String,
    pub entity_id: String,
    pub entity_type: String,
    pub change_type: ChangeType,
    pub changed_at: i64,
    pub changed_by: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDiff {
    pub before: serde_json::Value,
    pub after: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionTimeline {
    pub entity_id: String,
    pub revisions: Vec<MemoryRevision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub timestamp: i64,
    pub entities: Vec<String>,
}
