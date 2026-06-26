use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupersessionRecord {
    pub old_node: String,
    pub replacement_node: String,
    pub reason: String,
    pub confidence: f32,
}

impl SupersessionRecord {
    pub fn new(
        old_node: String,
        replacement_node: String,
        reason: String,
        confidence: f32,
    ) -> Self {
        Self {
            old_node,
            replacement_node,
            reason,
            confidence,
        }
    }
}
