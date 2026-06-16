use crate::models::ResolutionPriority;
use ares_gap_engine::models::{Gap, GapSeverity, KnowledgeDebt, RepositoryHealthSnapshot};

pub struct ResolutionPrioritizer;

impl ResolutionPrioritizer {
    pub fn new() -> Self {
        Self
    }

    pub fn prioritize(
        &self,
        gap: &Gap,
        _health: &RepositoryHealthSnapshot,
        _debt: &KnowledgeDebt,
    ) -> ResolutionPriority {
        // Base priority on gap severity
        let mut score = match gap.severity {
            GapSeverity::Critical => 80.0,
            GapSeverity::Warning => 50.0,
            GapSeverity::Info => 20.0,
        };

        // Escalate based on ImpactRadius
        if let Some(radius) = &gap.impact_radius {
            let total_impact = radius.requirements + radius.decisions + radius.architecture_components + radius.code_artifacts;
            
            if total_impact > 20 {
                score += 30.0;
            } else if total_impact > 10 {
                score += 20.0;
            } else if total_impact > 5 {
                score += 10.0;
            }
        }

        if score >= 90.0 {
            ResolutionPriority::Critical
        } else if score >= 70.0 {
            ResolutionPriority::High
        } else if score >= 40.0 {
            ResolutionPriority::Medium
        } else {
            ResolutionPriority::Low
        }
    }
}

impl Default for ResolutionPrioritizer {
    fn default() -> Self {
        Self::new()
    }
}
