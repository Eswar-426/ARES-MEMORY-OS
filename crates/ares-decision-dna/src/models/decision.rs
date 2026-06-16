use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ImpactMap, ProvenanceRecord, ReasoningChain};

pub type DecisionId = Uuid;
pub type UserId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DecisionState {
    Proposed,
    Accepted,
    Rejected,
    Superseded,
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReviewTrigger {
    TimeElapsed,
    AssumptionInvalidated,
    ContradictionDetected,
    ImpactedFilesChanged,
    RelatedBugCreated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionMemory {
    pub id: DecisionId,
    pub title: String,
    pub context: String,
    pub state: DecisionState,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // AI and Review Metadata
    pub confidence: f32,
    pub ai_assisted: bool,
    pub human_reviewed: bool,
    pub review_due_at: Option<DateTime<Utc>>,
    pub approved_by: Vec<UserId>,
    pub tags: Vec<String>,

    // Historical Evolution
    pub supersedes: Vec<DecisionId>,
    pub superseded_by: Option<DecisionId>,

    pub provenance: ProvenanceRecord,
    pub reasoning: ReasoningChain,
    pub impact: ImpactMap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOutcome {
    pub decision_id: DecisionId,
    pub success_score: f32,
    pub lessons_learned: Vec<String>,
    pub measured_at: DateTime<Utc>,
}
