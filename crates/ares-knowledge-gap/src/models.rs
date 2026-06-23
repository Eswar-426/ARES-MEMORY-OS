use ares_core::types::node::NodeType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KnowledgeGapType {
    MissingRequirement,
    MissingDecision,
    MissingArchitecture,
    MissingOwnership,
    MissingTests,
    MissingRuntimeValidation,
    MissingOutcomeTracking,
    MissingTraceability,
    KnowledgeBlindSpot,
    KnowledgeDebt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GapSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapEvidence {
    pub source_nodes: Vec<String>,
    pub missing_nodes: Vec<NodeType>,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationRecommendation {
    pub priority: GapSeverity,
    pub owner: Option<String>,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGap {
    pub gap_type: KnowledgeGapType,
    pub severity: GapSeverity,
    pub evidence: GapEvidence,
    pub remediation: RemediationRecommendation,
}
