use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningResponse {
    pub conclusion: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub path: Vec<Uuid>,
}
