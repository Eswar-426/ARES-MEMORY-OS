use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArchitectureCategory {
    Service,
    Component,
    Module,
    Workspace,
    Domain,
    Integration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TraceabilityCategory {
    RequirementToDecision,
    DecisionToArchitecture,
    ArchitectureToCode,
    RequirementToCode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TraceabilityEndpointType {
    Candidate,
    GraphNode,
    File,
    Commit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraceabilityEndpoint {
    pub endpoint_type: TraceabilityEndpointType,
    pub endpoint_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TraceabilityStrength {
    Weak,
    Moderate,
    Strong,
    Definitive,
}

pub use crate::confidence::CandidateConfidence;
pub use crate::promotion::{CandidatePromotion, CandidateReview};
pub use crate::sources::CandidateSource;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Candidate {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub candidate_type: CandidateType,
    pub decision_category: Option<DecisionCategory>,
    pub architecture_category: Option<ArchitectureCategory>,
    pub traceability_category: Option<TraceabilityCategory>,
    pub source_endpoint: Option<TraceabilityEndpoint>,
    pub target_endpoint: Option<TraceabilityEndpoint>,
    pub traceability_strength: Option<TraceabilityStrength>,
    pub ownership_domains: Vec<String>,
    pub dependent_components: Vec<String>,
    pub status: CandidateStatus,
    pub confidence: CandidateConfidence,
    pub created_at: i64,
    pub updated_at: i64,
}
