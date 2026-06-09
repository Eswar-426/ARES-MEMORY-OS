use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteEntityCommand {
    pub entity_id: String,
    pub reason: String,
    pub source: String,
}
