use ares_core::{
    ArchComponentId, CodeArtifactId, DecisionId, EvidenceId, ProjectId, RequirementId,
    RequirementLinkId, RuntimeMetricId,
};
use serde::{Deserialize, Serialize};

/// Core requirement entity.
/// Relationships live exclusively in `requirement_links` — never on this struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: RequirementId,
    pub project_id: ProjectId,
    pub title: String,
    pub description: String,
    pub source: RequirementSource,
    pub requirement_type: RequirementType,
    pub status: RequirementStatus,
    pub priority: RequirementPriority,
    pub owner: Option<String>,
    pub created_at: i64, // Unix microseconds
    pub updated_at: i64, // Unix microseconds
    pub tags: Vec<String>,
}

#[derive(utoipa::ToSchema)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementSource {
    Customer,
    Product,
    Compliance,
    Security,
    Architecture,
    TechnicalDebt,
}

#[derive(utoipa::ToSchema)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementType {
    Functional,
    NonFunctional,
    Security,
    Performance,
    Compliance,
    Architecture,
    Business,
    TechnicalDebt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementStatus {
    Draft,
    Proposed,
    Approved,
    Implemented,
    Verified,
    Deprecated,
    Rejected,
}

#[derive(utoipa::ToSchema)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(utoipa::ToSchema)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricProvider {
    Prometheus,
    OpenTelemetry,
    Datadog,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMetricRef {
    pub id: RuntimeMetricId,
    pub provider: MetricProvider,
    pub external_id: Option<String>,
}

/// Every entity in the traceability chain is addressable via a canonical target.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "id", rename_all = "snake_case")]
pub enum LinkTarget {
    Requirement(RequirementId),
    Decision(DecisionId),
    Architecture(ArchComponentId),
    Code(CodeArtifactId),
    RuntimeMetric(RuntimeMetricRef),
}

impl LinkTarget {
    pub fn target_type(&self) -> LinkTargetType {
        match self {
            Self::Requirement(_) => LinkTargetType::Requirement,
            Self::Decision(_) => LinkTargetType::Decision,
            Self::Architecture(_) => LinkTargetType::Architecture,
            Self::Code(_) => LinkTargetType::Code,
            Self::RuntimeMetric(_) => LinkTargetType::RuntimeMetric,
        }
    }

    pub fn target_id(&self) -> &str {
        match self {
            Self::Requirement(id) => id.as_str(),
            Self::Decision(id) => id.as_str(),
            Self::Architecture(id) => id.as_str(),
            Self::Code(id) => id.as_str(),
            Self::RuntimeMetric(r) => r.id.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkTargetType {
    Requirement,
    Decision,
    Architecture,
    Code,
    RuntimeMetric,
}

/// Cross-entity relationships (Requirement → Decision, Architecture, Code)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkRelationship {
    ApprovedBy,
    ImplementedBy,
    VerifiedBy,
    MonitoredBy,
    ResultsIn,
    OwnedBy,
    Drives,
    Implements,
    TracesTo,
    Validates,
}

/// Intra-requirement relationships (Requirement → Requirement)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementRelationship {
    DependsOn,
    Blocks,
    ParentOf,
    ChildOf,
    Supersedes,
    DerivedFrom,
}

pub struct RequirementLink {
    pub id: RequirementLinkId,
    pub source_requirement_id: RequirementId,
    pub target: LinkTarget,
    pub relationship: String, // Serialized from LinkRelationship or RequirementRelationship
    pub created_at: i64,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRequirementInput {
    pub project_id: ProjectId,
    pub title: String,
    pub description: String,
    pub source: RequirementSource,
    pub requirement_type: RequirementType,
    pub priority: RequirementPriority,
    pub owner: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRequirementInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub source: Option<RequirementSource>,
    pub requirement_type: Option<RequirementType>,
    pub status: Option<RequirementStatus>,
    pub priority: Option<RequirementPriority>,
    pub owner: Option<Option<String>>, // Some(None) = clear owner
    pub tags: Option<Vec<String>>,
    pub change_reason: Option<String>, // Fed into revision history
    pub changed_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequirementFilter {
    pub status: Option<RequirementStatus>,
    pub priority: Option<RequirementPriority>,
    pub requirement_type: Option<RequirementType>,
    pub owner: Option<String>,
    pub tag: Option<String>,
    pub since: Option<i64>,
    pub until: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSourceType {
    Jira,
    Adr,
    Rfc,
    Commit,
    Pr,
    Issue,
    Document,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementEvidence {
    pub id: EvidenceId,
    pub requirement_id: RequirementId,
    pub source_type: EvidenceSourceType,
    pub source_reference: String,
    pub confidence_score: f32,
    pub created_at: i64,
}

#[derive(utoipa::ToSchema)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementSummary {
    pub id: String,
    pub title: String,
    pub status: RequirementStatus,
    pub priority: RequirementPriority,
    pub requirement_type: RequirementType,
    pub owner: Option<String>,
    pub link_count: usize,
    pub created_at: i64,
}

#[derive(utoipa::ToSchema)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StructuralDrift {
    MissingDecision,
    MissingImplementation,
    MissingVerification,
    MissingMonitoring,
    MissingOwner,
}

#[derive(utoipa::ToSchema)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SemanticDrift {
    DecisionChanged,
    ImplementationChanged,
    RuntimeMismatch,
    RequirementExpired,
}

#[derive(utoipa::ToSchema)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RequirementDriftType {
    Structural(StructuralDrift),
    Semantic(SemanticDrift),
}

#[derive(utoipa::ToSchema)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DriftSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(utoipa::ToSchema)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DriftConfidence {
    Certain,
    High,
    Medium,
    Low,
}

#[derive(utoipa::ToSchema)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftEvidence {
    pub source_node: String,
    pub target_node: String,
    pub relationship: String,
    pub observed_state: String,
    pub expected_state: String,
}

#[derive(utoipa::ToSchema)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementDriftReport {
    pub requirement_id: String,
    pub severity: DriftSeverity,
    pub drift_types: Vec<RequirementDriftType>,
    pub evidence: Vec<DriftEvidence>,
    pub confidence: DriftConfidence,
    pub explanations: Vec<String>,
    pub remediations: Vec<String>,
}

#[derive(utoipa::ToSchema)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementBaseline {
    pub requirement_id: String,
    pub approved_at: i64,
    pub decision_ids: Vec<String>,
    pub implementation_ids: Vec<String>,
    pub test_ids: Vec<String>,
    pub runtime_metrics: Vec<RuntimeMetricRef>,
}
