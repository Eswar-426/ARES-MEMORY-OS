use ares_core::{
    ArchComponentId, CodeArtifactId, DecisionId, EvidenceId, ProjectId, RequirementId,
    RequirementLinkId,
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
    Approved,
    Implemented,
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

/// Every entity in the traceability chain is addressable via a canonical target.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "id", rename_all = "snake_case")]
pub enum LinkTarget {
    Requirement(RequirementId),
    Decision(DecisionId),
    Architecture(ArchComponentId),
    Code(CodeArtifactId),
}

impl LinkTarget {
    pub fn target_type(&self) -> LinkTargetType {
        match self {
            Self::Requirement(_) => LinkTargetType::Requirement,
            Self::Decision(_) => LinkTargetType::Decision,
            Self::Architecture(_) => LinkTargetType::Architecture,
            Self::Code(_) => LinkTargetType::Code,
        }
    }

    pub fn target_id(&self) -> &str {
        match self {
            Self::Requirement(id) => id.as_str(),
            Self::Decision(id) => id.as_str(),
            Self::Architecture(id) => id.as_str(),
            Self::Code(id) => id.as_str(),
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
}

/// Cross-entity relationships (Requirement → Decision, Architecture, Code)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkRelationship {
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
    pub requirement_type: RequirementType,
    pub priority: RequirementPriority,
    pub owner: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRequirementInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub requirement_type: Option<RequirementType>,
    pub status: Option<RequirementStatus>,
    pub priority: Option<RequirementPriority>,
    pub owner: Option<Option<String>>, // Some(None) = clear owner
    pub tags: Option<Vec<String>>,
    pub change_reason: Option<String>, // Fed into revision history
    pub changed_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
