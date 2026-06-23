use crate::models::DecisionStatus;
use serde::{Deserialize, Serialize};

#[derive(utoipa::ToSchema, Debug, Clone, Serialize, Deserialize)]
pub struct DecisionSummary {
    pub id: String,
    pub title: String,
    pub approval_status: DecisionStatus,
}

#[derive(utoipa::ToSchema, Debug, Clone, Serialize, Deserialize)]
pub struct DecisionCoverage {
    pub total_decisions: usize,
    pub approved_decisions: usize,
    pub decisions_with_consequences: usize,
    pub decisions_with_evidence: usize,
}
