use super::capability::ModelCapability;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: Uuid,
    pub name: String,
    pub provider_id: String,
    pub version: String,
    pub capabilities: Vec<ModelCapability>,
    pub max_context_window: usize,
    pub cost_per_1k_tokens: f64,
}
