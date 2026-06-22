use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CandidateStatus {
    Proposed,
    UnderReview,
    Approved,
    Rejected,
    Superseded,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CandidateType {
    Requirement,
    Decision,
    Architecture,
    Traceability,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DecisionCategory {
    TechnologyAdoption,
    TechnologyRemoval,
    DependencyMigration,
    ArchitectureChange,
    PlatformChoice,
}

pub use crate::confidence::CandidateConfidence;
pub use crate::sources::CandidateSource;
pub use crate::promotion::{CandidatePromotion, CandidateReview};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub candidate_type: CandidateType,
    pub decision_category: Option<DecisionCategory>,
    pub status: CandidateStatus,
    pub confidence: CandidateConfidence,
    pub created_at: i64,
    pub updated_at: i64,
}
