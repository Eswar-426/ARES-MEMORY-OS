use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ImpactSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangeImpactReport {
    pub target_node_id: String,
    pub project_id: String,

    pub impacted_requirements: Vec<String>,
    pub impacted_decisions: Vec<String>,
    pub impacted_architecture: Vec<String>,
    pub impacted_files: Vec<String>,
    pub impacted_owners: Vec<String>,

    pub drift_risk: f32,         // 0-100
    pub staleness_risk: f32,     // 0-100
    pub total_impact_score: f32, // 0-100

    pub severity: ImpactSeverity,
    pub rationale: Vec<String>,
}
