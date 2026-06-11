use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionFeedback {
    pub execution_id: Uuid,
    pub model_id: Uuid,
    pub success: bool,
    pub quality_score: Option<f64>,
    pub hallucination_detected: bool,
}
