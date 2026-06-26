pub mod assumption;
pub mod conflict;
pub mod decision_dna;
pub mod review_trigger;
pub mod risk;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, utoipa::ToSchema)]
pub enum DecisionStatus {
    Proposed,
    Approved,
    Rejected,
    Deprecated,
}

impl DecisionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "Proposed",
            Self::Approved => "Approved",
            Self::Rejected => "Rejected",
            Self::Deprecated => "Deprecated",
        }
    }
}

pub use assumption::AssumptionNode;
pub use conflict::{ConflictType, DecisionConflict};
pub use decision_dna::DecisionDNA;
pub use review_trigger::ReviewTriggerNode;
pub use risk::RiskNode;

// Legacy Test Shims
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Decision {
    pub id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionConfidence {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionEvidence {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionOutcome {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EvidenceSource {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OutcomeType {}
