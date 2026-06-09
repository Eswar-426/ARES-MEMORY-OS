use crate::models::candidate::PlanCandidate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanExplanation {
    pub plan_id: ares_core::id::PlanId,
    pub reasoning: String,
    pub alternative_candidates_considered: usize,
    pub primary_objective: String,
}

pub struct ExplainabilityEngine;

impl ExplainabilityEngine {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    /// Generates a human-readable explanation for why a particular plan candidate was selected.
    pub fn generate_explanation(
        &self,
        plan_id: &ares_core::id::PlanId,
        selected: &PlanCandidate,
        all_candidates: &[PlanCandidate],
        objective: &str,
    ) -> PlanExplanation {
        let reasoning = format!(
            "Plan selected because it scored highest ({:.2}) for objective '{}'. \
            Expected cost: ${:.2}, Duration: {:.0}s.",
            selected.score,
            objective,
            selected.estimated_cost.unwrap_or(0.0),
            selected.estimated_duration.unwrap_or(0.0)
        );

        PlanExplanation {
            plan_id: plan_id.clone(),
            reasoning,
            alternative_candidates_considered: all_candidates.len().saturating_sub(1),
            primary_objective: objective.to_string(),
        }
    }
}
