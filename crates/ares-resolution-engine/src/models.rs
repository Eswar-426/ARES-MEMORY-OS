use serde::{Deserialize, Serialize};
use ares_gap_engine::models::RootCause;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EffortLevel {
    Trivial,
    Small,
    Medium,
    Large,
    Strategic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ResolutionCategory {
    Governance,
    Traceability,
    Documentation,
    Validation,
    Ownership,
    Architecture,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ResolutionConfidence {
    Guaranteed,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResolutionPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ResolutionActionType {
    CreateRequirement,
    CreateDecision,
    CreateArchitectureLink,
    CreateCodeLink,
    AssignOwner,
    AddEvidence,
    AddApproval,
    AddOutcome,
    AddConsequences,
    CreateTraceabilityLink,
    CreateValidation,
    ArchiveEntity,
    UpdateDocumentation,
    ReviewEntity,
    GovernanceReview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolutionImpact {
    pub entities_resolved: usize,
    pub severity_reduction: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolutionAction {
    pub id: String,
    pub title: String,
    pub description: String,
    pub action_type: ResolutionActionType,
    pub target_entities: Vec<String>,
    pub expected_impact: ResolutionImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolutionTemplate {
    pub id: String,
    pub title: String,
    pub actions: Vec<ResolutionActionType>,
    pub expected_health_gain: f64,
    pub expected_debt_reduction: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthGainBreakdown {
    pub ownership_gain: f64,
    pub traceability_gain: f64,
    pub governance_gain: f64,
    pub validation_gain: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolutionPlan {
    pub id: String,
    pub gap_id: String,
    pub root_cause: RootCause,
    pub category: ResolutionCategory,
    pub confidence: ResolutionConfidence,
    pub actions: Vec<ResolutionAction>,
    pub priority: ResolutionPriority,
    pub estimated_effort: EffortLevel,
    pub expected_health_gain: f64,
    pub expected_debt_reduction: f64,
    pub health_gain_breakdown: HealthGainBreakdown,
    pub generated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolutionReport {
    pub repository_health: f64,
    pub knowledge_debt: f64,
    pub critical_gaps: usize,
    pub recommended_plans: Vec<ResolutionPlan>,
}
