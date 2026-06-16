use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{AssumptionTracking, RiskAssessment};

pub type ChainId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningChain {
    pub id: ChainId,
    pub steps: Vec<String>, // Structured JSON or text representation
    pub alternatives: Vec<AlternativeAnalysis>,
    pub assumptions: Vec<AssumptionTracking>,
    pub risks: Vec<RiskAssessment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeAnalysis {
    pub description: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub rejection_reason: String,
}
