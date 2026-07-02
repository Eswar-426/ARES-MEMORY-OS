use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// EngineeringQuery — the universal query context
// ═══════════════════════════════════════════════════════════════════

/// What kind of intelligence question is being asked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
    WhyExists,
    Impact,
    Traceability,
    Ownership,
    Drift,
    Coverage,
    Simulation,
}

/// The universal input for all intelligence queries.
///
/// Every intelligence feature receives the same context. Adding fields
/// here never breaks existing generators — they simply ignore what
/// they don't need.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineeringQuery {
    pub entity_id: String,
    pub project_id: String,
    pub query_type: QueryType,
    /// Workspace root path (e.g. `E:\My Projects\youtube-automation`)
    pub workspace_root: Option<String>,
    /// Git branch name (e.g. `main`)
    pub branch: Option<String>,
}

impl EngineeringQuery {
    pub fn with_type(mut self, query_type: QueryType) -> Self {
        self.query_type = query_type;
        self
    }
}
