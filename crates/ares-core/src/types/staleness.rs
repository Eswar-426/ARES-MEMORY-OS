use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StalenessFactors {
    pub age_days: u32,
    pub downstream_changes: u32,
    pub dependent_nodes: u32,
    pub ownership_changes: u32,
    pub evolution_events: u32,
}

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum HealthClassification {
    Healthy,
    Aging,
    Stale,
    Critical,
}

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryHealthScore {
    pub score: f32, // 0-100
    pub classification: HealthClassification,
    pub factors: StalenessFactors,
}

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StalenessFinding {
    pub node_id: String,
    pub project_id: String,
    pub score: f32,
    pub classification: HealthClassification,
    pub rationale: Vec<String>,
}
