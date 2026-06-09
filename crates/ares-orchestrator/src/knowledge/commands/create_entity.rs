use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEntityCommand {
    pub entity_id: String,
    pub entity_type: String,
    pub attributes: serde_json::Value,
    pub source: String,
}
