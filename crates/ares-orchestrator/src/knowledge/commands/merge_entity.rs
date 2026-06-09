use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeEntityCommand {
    pub target_entity_id: String,
    pub source_entity_id: String, // The entity being merged into the target
    pub resolution_strategy: String,
    pub source: String,
}
