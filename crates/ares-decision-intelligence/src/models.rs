use ares_core::id::{DecisionId, EvidenceId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: DecisionId,
    pub title: String,
    pub context: String,
    pub problem: String,
    pub chosen_option: String,
    pub rejected_options: Vec<DecisionAlternative>,
    pub assumptions: Vec<String>,
    pub consequences: Vec<DecisionConsequence>,
    pub confidence: DecisionConfidence,
    pub owner: Option<String>,
    pub approval_status: DecisionStatus,
    pub approved_by: Option<String>,
    pub approved_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionAlternative {
    pub title: String,
    pub description: String,
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionConsequence {
    pub description: String,
    pub consequence_type: ConsequenceType,
    pub severity: ConsequenceSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOutcome {
    pub id: String, // Or OutcomeId if defined in ares_core
    pub decision_id: DecisionId,
    pub observed_at: i64,
    pub description: String,
    pub outcome_type: OutcomeType,
    pub success_score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Superseded,
    Deprecated,
}

impl DecisionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
            Self::Deprecated => "deprecated",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConsequenceType {
    Positive,
    Negative,
    Risk,
    Tradeoff,
    Cost,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConsequenceSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeType {
    Success,
    Failure,
    Partial,
    Unexpected,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionConfidence {
    Experimental,
    Low,
    Medium,
    High,
    Proven,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    ADR,
    RFC,
    PullRequest,
    GitCommit,
    Issue,
    DesignDocument,
    Benchmark,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEvidence {
    pub id: EvidenceId,
    pub decision_id: DecisionId,
    pub source: EvidenceSource,
    pub reference_url: String,
    pub description: String,
    pub confidence_score: f32,
}
