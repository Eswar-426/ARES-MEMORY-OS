use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TopologyState {
    Complete,
    Partial,
    Orphaned,
    Disconnected,
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchySegment {
    pub node_id: String,
    pub node_type: String,
    pub state: TopologyState,
    pub missing_downstream: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageMetrics {
    pub requirement_coverage: f32,  // % of Req -> Dec
    pub decision_coverage: f32,     // % of Dec -> Arch
    pub architecture_coverage: f32, // % of Arch -> Code
    pub code_coverage: f32,         // % of Code -> Test
    pub test_coverage: f32,         // % of Test -> Runtime
    pub runtime_coverage: f32,      // % of Runtime -> Outcome
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageSnapshot {
    pub id: String,
    pub project_id: String,
    pub timestamp: DateTime<Utc>,
    pub metrics: CoverageMetrics,
    pub overall_coverage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoverageDimension {
    Repository,
    Team,
    Capability,
    Domain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryHealthScore {
    pub project_id: String,
    pub coverage_score: f32,     // 35%
    pub completeness_score: f32, // 35%
    pub traceability_score: f32, // 20%
    pub staleness_score: f32,    // 10%
    pub total_health: f32,
    pub snapshot_date: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GapPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizedGap {
    pub node_id: String,
    pub gap_description: String,
    pub priority: GapPriority,
    pub total_risk_score: f32,
    pub impact_score: f32,
    pub drift_score: f32,
    pub staleness_score: f32,
    pub reachability_score: f32,
}
