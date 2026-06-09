use ares_core::id::AgentId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedCapability {
    pub name: String,
    pub description: String,
    pub required_inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub provider_agent_id: AgentId,
    pub cached_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedModel {
    pub model_id: String,
    pub max_tokens: u32,
    pub cost_per_1k_tokens: f64,
    pub cached_at: DateTime<Utc>,
}
