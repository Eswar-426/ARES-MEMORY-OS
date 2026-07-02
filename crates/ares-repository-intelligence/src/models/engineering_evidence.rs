use super::evidence_types::{
    ContributorRef, DependencyRef, EntityMetrics, EntityRef, GitEvidence, RiskLevel, Timestamps,
};
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// EngineeringEvidence — the rich, shared data container
// ═══════════════════════════════════════════════════════════════════

/// All known engineering facts about a single entity.
///
/// Fields are **semantically categorized** — generators never inspect
/// raw graph edges. They ask for `dependencies`, `contributors`,
/// `folders`, etc., and the EvidenceService has already done the
/// classification work.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineeringEvidence {
    // ── Identity ─────────────────────────────────────────────────
    pub entity_id: String,
    pub entity_type: String,
    pub entity_label: String,
    pub file_path: Option<String>,
    pub project_id: String,

    // ── Hierarchy ────────────────────────────────────────────────
    /// Folders and packages that contain this entity.
    pub folders: Vec<EntityRef>,
    /// The immediate parent module, if any.
    pub parent_module: Option<EntityRef>,

    // ── Code Dependencies ────────────────────────────────────────
    /// What this entity imports, calls, or depends on.
    pub dependencies: Vec<DependencyRef>,
    /// What imports, calls, or depends on this entity.
    pub dependents: Vec<DependencyRef>,

    // ── People ───────────────────────────────────────────────────
    /// People who authored commits touching this entity, with counts.
    pub contributors: Vec<ContributorRef>,
    /// Declared code owners.
    pub owners: Vec<String>,

    // ── History ──────────────────────────────────────────────────
    pub commits: Vec<GitEvidence>,

    // ── Architecture ─────────────────────────────────────────────
    pub requirements: Vec<String>,
    pub decisions: Vec<String>,

    // ── Code Structure ───────────────────────────────────────────
    pub symbols: Vec<String>,
    pub references: Vec<String>,
    pub tests: Vec<String>,
    pub documentation: Vec<String>,

    // ── Metrics & Risk ───────────────────────────────────────────
    pub metrics: Option<EntityMetrics>,
    pub timestamps: Option<Timestamps>,
    pub risk: Option<RiskLevel>,
}

impl EngineeringEvidence {
    pub fn not_found(entity_id: &str, project_id: &str) -> Self {
        Self {
            entity_id: entity_id.to_string(),
            entity_type: "unknown".to_string(),
            entity_label: entity_id.to_string(),
            file_path: None,
            project_id: project_id.to_string(),
            folders: Vec::new(),
            parent_module: None,
            dependencies: Vec::new(),
            dependents: Vec::new(),
            contributors: Vec::new(),
            owners: Vec::new(),
            commits: Vec::new(),
            requirements: Vec::new(),
            decisions: Vec::new(),
            symbols: Vec::new(),
            references: Vec::new(),
            tests: Vec::new(),
            documentation: Vec::new(),
            metrics: None,
            timestamps: None,
            risk: None,
        }
    }
}
