use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRelationshipCommand {
    pub source_entity_id: String,
    pub target_entity_id: String,
    pub relationship_type: String,
    pub attributes: serde_json::Value,
    pub source: String,
}
