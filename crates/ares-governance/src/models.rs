use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Default)]
pub enum EnforcementAction {
    Allow,
    #[default]
    Warn,
    RequireApproval,
    Block,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum MemoryRiskLevel {
    MemorySafe,
    MemoryWarning,
    MemoryRisk,
    MemoryCritical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum CertificationLevel {
    None,
    Bronze,
    Silver,
    Gold,
    Platinum,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Default)]
pub enum PolicyCategory {
    Ownership,
    Approval,
    Traceability,
    Evidence,
    Retention,
    Security,
    #[default]
    Architecture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyVersion {
    pub policy_name: String,
    pub version: String,
    pub checksum: String,
    pub loaded_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceViolation {
    pub id: String,
    pub severity: ViolationSeverity,
    pub policy_name: String,
    pub node_id: String,
    pub reason: String,
    pub supporting_nodes: Vec<String>,
    #[serde(default)]
    pub enforcement: EnforcementAction,
    #[serde(default)]
    pub category: PolicyCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceResult {
    pub id: String,
    pub project_id: String,
    pub entity_id: String,
    pub policy_version_id: String,
    pub compliant: bool,
    pub score: f32,
    pub evaluated_at: i64,
    pub violations: Vec<ComplianceViolation>,
    #[serde(default)]
    pub category: PolicyCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GovernanceExplanationSummary {
    pub id: String,
    pub title: String,
    pub reason: String,
    pub missing: String,
    pub owner: String,
    pub fix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GovernanceExplanation {
    pub summary: GovernanceExplanationSummary,
    pub evidence: Vec<String>,
    pub policy_id: String,
    pub severity: ViolationSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceScorecard {
    pub ownership_score: f32,
    pub traceability_score: f32,
    pub evidence_score: f32,
    pub approval_score: f32,
    pub retention_score: f32,
    pub security_score: f32,
    pub architecture_score: f32,
    pub overall_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GovernanceCertification {
    pub id: String,
    pub project_id: String,
    pub certified: bool,
    pub policy_score: f32,
    pub level: CertificationLevel,
    pub violations_count: usize,
    pub scorecard: GovernanceScorecard,
    pub evaluated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDefinition {
    pub api_version: String,
    pub kind: String,
    pub metadata: PolicyMetadata,
    pub spec: PolicySpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub category: PolicyCategory,
    #[serde(default)]
    pub enforcement: EnforcementAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySpec {
    pub rules: Vec<PolicyRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub name: String,
    pub severity: ViolationSeverity,
    pub condition: String,
    pub target: Vec<String>,
    #[serde(default)]
    pub enforcement: EnforcementAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EnforcementDecision {
    pub allowed: bool,
    pub action: EnforcementAction,
    pub violations: Vec<ComplianceViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PolicyExemption {
    pub id: String,
    pub reason: String,
    pub approved_by: String,
    pub approved_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub target_rules: Vec<String>,
    #[serde(default)]
    pub target_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EnforcementReadiness {
    pub ready: bool,
    pub blocking_violations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PolicyDriftStatus {
    pub project_id: String,
    pub drift_detected: bool,
    pub outdated_policies: Vec<String>,
}

use ares_requirements::{GapSummary, RequirementCoverageSummary, RequirementCoverageTrend};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DecisionHealthMetrics {
    pub total_decisions: usize,
    pub active_decisions: usize,
    pub stale_decisions: usize,
    pub expired_decisions: usize,
    pub orphan_decisions: usize,
    pub health_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct KnowledgeDebtMetrics {
    pub orphan_requirements: usize,
    pub orphan_decisions: usize,
    pub missing_owners: usize,
    pub missing_evidence: usize,
    pub traceability_gaps: usize,
    pub policy_violations: usize,
    pub debt_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApprovalMetrics {
    pub pending: usize,
    pub approved_today: usize,
    pub rejected_today: usize,
    pub expired: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DriftMetrics {
    pub drift_detected: bool,
    pub changed_policies: usize,
    pub unevaluated_policies: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RequirementDriftSummary {
    pub structural_drift: usize,
    pub semantic_drift: usize,
    pub critical_drift: usize,
    pub unresolved_drift: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EvolutionMetrics {
    pub total_requirement_events: usize,
    pub requirements_changed_this_week: usize,
    pub requirements_regressed: usize,
    pub requirements_improved: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GovernanceDashboard {
    pub certification: GovernanceCertification,
    pub scorecard: GovernanceScorecard,
    pub decision_health: DecisionHealthMetrics,
    pub knowledge_debt: KnowledgeDebtMetrics,
    pub approvals: ApprovalMetrics,
    pub compliance_drift: DriftMetrics,
    pub requirement_coverage: RequirementCoverageSummary,
    pub requirement_coverage_trend: RequirementCoverageTrend,
    pub requirement_drift: RequirementDriftSummary,
    pub evolution: EvolutionMetrics,
    pub top_gaps: Vec<GapSummary>,
    pub top_violations: Vec<ComplianceViolation>,
}

// --- Phase 8H: Runtime Enforcement Models ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GovernanceOutcome {
    Allow,
    Warn,
    RequireApproval,
    Block,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GovernanceApprovalRequest {
    pub id: String,
    pub workflow_id: String,
    pub project_id: String,
    pub violations: Vec<ComplianceViolation>,
    pub status: ApprovalStatus,
    pub requested_by: String,
    pub approved_by: Option<String>,
    pub requested_at: i64,
    pub updated_at: i64,
    pub expires_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub enum GovernanceEvent {
    ViolationDetected {
        project_id: String,
        workflow_id: String,
        violations: Vec<ComplianceViolation>,
    },
    ApprovalRequested {
        request: GovernanceApprovalRequest,
    },
    Approved {
        request_id: String,
        approved_by: String,
        approved_at: i64,
    },
    Rejected {
        request_id: String,
        rejected_by: String,
        rejected_at: i64,
        reason: String,
    },
    Blocked {
        project_id: String,
        workflow_id: String,
        reason: String,
        violations: Vec<ComplianceViolation>,
    },
}
