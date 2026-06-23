use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftType {
    CapabilityMismatch,
    DependencyMismatch,
    OwnershipMismatch,
    ConfigurationMismatch,
    ArchitectureMismatch,
    TraceabilityGap,
}

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftCandidate {
    pub id: String,
    pub project_id: String,
    pub target_node_id: String,
    pub drift_type: DriftType,
    pub confidence: f32,
    pub evidence_ids: Vec<String>,
    pub rationale: String,
    pub detected_at: DateTime<Utc>,
}
