use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSyncEvent {
    pub entity_id: String,
    pub payload: String,
    pub confidence: f64,
}
