use ares_core::id::ProjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gap {
    pub id: String,
    pub project_id: ProjectId,
    pub gap_type: GapType,
    pub description: String,

    /// Target entity where the gap was discovered (e.g. RequirementId, DecisionId)
    pub source_id: String,

    /// The specific method that flagged this gap.
    pub detection_method: DetectionMethod,

    /// A computed score (0.0 - 1.0) indicating the likelihood or severity of the gap,
    /// rather than a subjective "confidence" score.
    pub evidence_score: f32,

    pub severity: GapSeverity,
    pub identified_at: i64,

    /// Contextual metadata regarding the gap
    pub metadata: HashMap<String, String>,

    // Phase 3 Intelligence fields
    pub evidence: Vec<GapEvidence>,
    pub reason: Option<GapReason>,
    pub priority_score: Option<PriorityScore>,
    pub impact_radius: Option<ImpactRadius>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapEvidence {
    pub source_entity: String,
    pub source_type: String,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RootCause {
    GovernanceFailure,
    TraceabilityBreakdown,
    OwnershipFailure,
    DocumentationFailure,
    ValidationFailure,
    ProcessDrift,
    MemoryDecay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapReason {
    pub root_cause: RootCause,
    pub explanation: String,
    pub supporting_evidence: Vec<GapEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactRadius {
    pub requirements: usize,
    pub decisions: usize,
    pub architecture_components: usize,
    pub code_artifacts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityScore {
    pub score: f64,          // e.g. 0.0 to 100.0
    pub criticality: String, // e.g. "Low", "Medium", "High", "Critical"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapCluster {
    pub id: String,
    pub root_cause: RootCause,
    pub affected_entities: Vec<String>,
    pub severity: GapSeverity,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDebt {
    pub debt_score: f64,
    pub total_gaps: usize,
    pub critical_gaps: usize,
    pub stale_entities: usize,
    pub orphan_entities: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryHealthSnapshot {
    pub snapshot_id: String,
    pub project_id: ProjectId,
    pub snapshot_time: i64,
    pub overall_score: f64,
    pub component_scores: HashMap<String, f64>,
    pub total_gaps: usize,
    pub critical_gaps: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapType {
    /// Requirement is marked Implemented but lacks code traces
    MissingImplementation,

    /// Requirement exists but has no related architectural or technical decisions
    MissingDecision,

    /// A decision was made, but has no documented alternatives or evidence
    MissingEvidence,

    /// Code exists that doesn't trace back to any known requirement or decision
    OrphanCode,

    /// A decision is marked Approved but lacks an Owner
    MissingOwner,

    /// A requirement has not been updated in a long time
    StaleRequirement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionMethod {
    /// Hard rules (e.g., "if status == Approved && owner == None")
    Deterministic,

    /// Graph queries (e.g., "node has no incoming edges of type X")
    RuleBased,

    /// Statistical anomalies (e.g., "this file changed 50 times but has 0 decisions")
    Statistical,

    /// LLM-based detection (e.g., "the description of this decision conflicts with architecture")
    AIInference,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryHealthReport {
    pub project_id: ProjectId,
    pub generated_at: i64,

    pub health: RepositoryHealthSnapshot,
    pub gaps: Vec<Gap>,
    pub clusters: Vec<GapCluster>,
    pub prioritized_gaps: Vec<Gap>,
    pub knowledge_debt: KnowledgeDebt,
}
