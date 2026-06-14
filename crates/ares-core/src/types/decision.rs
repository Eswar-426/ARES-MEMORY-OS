use crate::id::{DecisionId, MemoryId, ProjectId};
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────
// Enumerations
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    Proposed,
    #[default]
    Accepted,
    Rejected,
    Superseded,
    Deprecated,
}

impl DecisionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
            Self::Deprecated => "deprecated",
        }
    }
}

impl std::str::FromStr for DecisionStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "proposed" => Ok(Self::Proposed),
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            "superseded" => Ok(Self::Superseded),
            "deprecated" => Ok(Self::Deprecated),
            other => Err(format!("Unknown decision status: {other}")),
        }
    }
}

// ─────────────────────────────────────────────────────────────────
// Sub-structures (embedded in Decision)
// ─────────────────────────────────────────────────────────────────

/// An alternative that was considered but not chosen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub option: String,
    pub description: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub reason_rejected: String,
    pub was_prototyped: bool,
}

/// A risk associated with the decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub risk: String,
    /// "low" | "medium" | "high" | "critical"
    pub severity: String,
    pub mitigation: String,
    pub accepted: bool,
}

/// Context captured at the moment the decision was made.
/// Captures assumptions that, if they change, may invalidate the decision.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub assumptions: Vec<String>,
    pub constraints_at_time: Vec<String>,
    pub unknown_factors: Vec<String>,
}

/// Predictions about future impact.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FutureImpact {
    pub if_technology_changes: Option<String>,
    pub if_team_scales: Option<String>,
    pub if_requirements_change: Option<String>,
    pub review_trigger_conditions: Vec<String>,
}

/// A single reasoning step in the chain that led to the decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    pub id: String,
    pub decision_id: DecisionId,
    pub step_order: u32,
    pub observation: String,
    pub inference: String,
    pub confidence: f32,
    pub created_at: i64,
}

// ─────────────────────────────────────────────────────────────────
// Decision DNA — core struct
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: DecisionId,
    pub project_id: ProjectId,
    /// Corresponds to a memory record (memory_type = "decision")
    pub memory_id: MemoryId,

    // Core content
    pub title: String,
    pub decision_text: String,
    pub reason: String,
    pub status: DecisionStatus,
    pub confidence: f32,

    // Reasoning chain (ordered by step_order)
    pub reasoning_steps: Vec<ReasoningStep>,

    // Alternatives and risks
    pub alternatives: Vec<Alternative>,
    pub risks: Vec<Risk>,

    // Context at decision time
    pub context_snapshot: ContextSnapshot,
    pub future_impact: FutureImpact,

    // Impact scope
    pub files_impacted: Vec<String>,
    pub services_impacted: Vec<String>,

    // Relationships
    pub supersedes: Vec<DecisionId>,
    pub superseded_by: Option<DecisionId>,

    // Provenance
    pub decided_by: String,
    pub discussed_in: Vec<String>,

    // Review
    pub review_due_at: Option<i64>,
    pub last_reviewed_at: Option<i64>,

    pub created_at: i64,
    pub updated_at: i64,
}

// ─────────────────────────────────────────────────────────────────
// Input / filter / patch types
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDecisionInput {
    pub project_id: ProjectId,
    pub title: String,
    pub memory_id: MemoryId,
    pub decision_text: String,
    pub reason: String,
    pub confidence: Option<f32>,
    pub alternatives: Option<Vec<Alternative>>,
    pub risks: Option<Vec<Risk>>,
    pub context_snapshot: Option<ContextSnapshot>,
    pub future_impact: Option<FutureImpact>,
    pub files_impacted: Option<Vec<String>>,
    pub services_impacted: Option<Vec<String>>,
    pub supersedes: Option<Vec<DecisionId>>,
    pub decided_by: Option<String>,
    pub discussed_in: Option<Vec<String>>,
    pub review_due_at: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecisionPatch {
    pub title: Option<String>,
    pub decision_text: Option<String>,
    pub reason: Option<String>,
    pub status: Option<DecisionStatus>,
    pub confidence: Option<f32>,
    pub alternatives: Option<Vec<Alternative>>,
    pub risks: Option<Vec<Risk>>,
    pub files_impacted: Option<Vec<String>>,
    pub services_impacted: Option<Vec<String>>,
    pub future_impact: Option<FutureImpact>,
    pub review_due_at: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecisionFilter {
    pub status: Option<DecisionStatus>,
    pub file_path: Option<String>,
    pub since: Option<i64>,
    pub until: Option<i64>,
    pub stale_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionSearchResult {
    pub decision: Decision,
    pub score: f64,
    pub snippet: String,
}
