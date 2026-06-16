use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EffortEstimation {
    Low,
    Medium,
    High,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactMap {
    pub files_affected: Vec<String>,
    pub systems_affected: Vec<String>,
    pub estimated_effort: EffortEstimation,
}
