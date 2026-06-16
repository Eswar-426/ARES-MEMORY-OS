use crate::models::{DecisionState, DecisionId, ProvenanceRecord, ImpactMap, ReasoningChain};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDecisionDto {
    pub title: String,
    pub context: String,
    pub confidence: f32,
    pub tags: Vec<String>,
    pub provenance: ProvenanceRecord,
    pub reasoning: ReasoningChain,
    pub impact: ImpactMap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResponseDto {
    pub id: DecisionId,
    pub title: String,
    pub state: DecisionState,
    pub version: u32,
    pub created_at: DateTime<Utc>,
}
