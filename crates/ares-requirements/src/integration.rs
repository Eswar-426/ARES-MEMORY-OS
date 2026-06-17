use crate::health::RequirementHealthScore;
use crate::gaps::GapSummary;
use crate::models::{RequirementPriority, RequirementStatus, RequirementType};
use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementCoverage {
    pub total_requirements: u64,
    pub approved_requirements: u64,
    pub implemented_requirements: u64,
    pub unlinked_requirements: u64,
    pub orphan_requirements: u64,
    pub coverage_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementHealthSnapshot {
    pub score: RequirementHealthScore,
    pub coverage: RequirementCoverage,
    pub gap_summary: GapSummary,
}
