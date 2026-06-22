use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::coverage_engine::MemoryCoverageMetrics;
use crate::memory_drift_engine::MemoryDriftMetrics;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MemoryHealthScore {
    pub coverage_score: f64,
    pub ownership_score: f64,
    pub evidence_score: f64,
    pub validation_score: f64,
    pub freshness_score: f64,
    pub total_health: f64,
}

pub struct MemoryHealthEngine;

impl MemoryHealthEngine {
    pub fn calculate(coverage: &MemoryCoverageMetrics, drift: &MemoryDriftMetrics) -> MemoryHealthScore {
        let coverage_pct = coverage.overall.percentage;
        let ownership_pct = coverage.ownership.percentage;
        let evidence_pct = coverage.evidence.percentage;
        let validation_pct = coverage.tests.percentage;
        let freshness_pct = 100.0 - drift.memory_drift_percentage;

        let coverage_score = coverage_pct * 0.35;
        let ownership_score = ownership_pct * 0.20;
        let evidence_score = evidence_pct * 0.15;
        let validation_score = validation_pct * 0.15;
        let freshness_score = freshness_pct * 0.15;

        let total_health = coverage_score
            + ownership_score
            + evidence_score
            + validation_score
            + freshness_score;

        MemoryHealthScore {
            coverage_score,
            ownership_score,
            evidence_score,
            validation_score,
            freshness_score,
            total_health,
        }
    }
}
