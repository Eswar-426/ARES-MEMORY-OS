use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssumptionTracking {
    pub statement: String,
    pub confidence_score: f32, // 0.0 - 1.0
    pub validation_strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskProbability {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub description: String,
    pub severity: RiskSeverity,
    pub probability: RiskProbability,
    pub mitigation: Option<String>,
    pub accepted: bool,
}
