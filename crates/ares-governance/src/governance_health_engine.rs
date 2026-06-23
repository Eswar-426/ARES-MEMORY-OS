use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceHealthScore {
    pub ownership_coverage_score: f64,
    pub approval_coverage_score: f64,
    pub governance_gap_score: f64,
    pub total_health: f64,
}

pub struct GovernanceHealthEngine;

impl GovernanceHealthEngine {
    pub fn calculate(
        ownership_coverage: f64,
        approval_coverage: f64,
        gap_score: f64,
    ) -> GovernanceHealthScore {
        let ownership_weight = 0.40;
        let approval_weight = 0.30;
        let gap_weight = 0.30;

        let total_health = (ownership_coverage * ownership_weight)
            + (approval_coverage * approval_weight)
            + (gap_score * gap_weight);

        GovernanceHealthScore {
            ownership_coverage_score: ownership_coverage,
            approval_coverage_score: approval_coverage,
            governance_gap_score: gap_score,
            total_health,
        }
    }
}
