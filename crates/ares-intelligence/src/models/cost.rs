use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostMetric {
    pub id: Uuid,
    pub model_id: Uuid,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub total_cost: f64,
}
