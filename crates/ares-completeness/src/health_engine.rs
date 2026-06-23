use crate::models::RepositoryHealthScore;
use chrono::Utc;

pub struct RepositoryHealthEngine;

impl Default for RepositoryHealthEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RepositoryHealthEngine {
    pub fn new() -> Self {
        Self
    }

    /// Calculates the Repository Health Score using weighted metrics.
    /// Formula:
    /// (Coverage * 0.35) + (Completeness * 0.35) + (Traceability * 0.20) + (Staleness * 0.10)
    pub fn calculate_health(
        &self,
        project_id: &str,
        coverage_score: f32,     // Expected 0.0 - 100.0
        completeness_score: f32, // Expected 0.0 - 100.0
        traceability_score: f32, // Expected 0.0 - 100.0
        staleness_score: f32,    // Expected 0.0 - 100.0
    ) -> RepositoryHealthScore {
        let total_health = (coverage_score * 0.35)
            + (completeness_score * 0.35)
            + (traceability_score * 0.20)
            + (staleness_score * 0.10);

        RepositoryHealthScore {
            project_id: project_id.to_string(),
            coverage_score,
            completeness_score,
            traceability_score,
            staleness_score,
            total_health: total_health.clamp(0.0, 100.0),
            snapshot_date: Utc::now(),
        }
    }
}
