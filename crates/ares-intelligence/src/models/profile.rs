use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    pub model_id: Uuid,
    pub success_rate: f64,
    pub average_latency_ms: u64,
    pub total_executions: u64,
}
