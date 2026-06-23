use crate::models::{DecisionId, DecisionMemory, DecisionState};
use chrono::Utc;

pub struct ReviewTriggerEngine;

impl ReviewTriggerEngine {
    pub fn check_time_elapsed(decisions: &[DecisionMemory]) -> Vec<DecisionId> {
        let now = Utc::now();
        decisions
            .iter()
            .filter(|d| d.state == DecisionState::Accepted)
            .filter(|d| {
                if let Some(due_at) = d.review_due_at {
                    due_at <= now
                } else {
                    false
                }
            })
            .map(|d| d.id)
            .collect()
    }

    pub fn check_impacted_files_changed(
        decisions: &[DecisionMemory],
        changed_files: &[String],
    ) -> Vec<DecisionId> {
        decisions
            .iter()
            .filter(|d| d.state == DecisionState::Accepted)
            .filter(|d| {
                d.impact
                    .files_affected
                    .iter()
                    .any(|file| changed_files.contains(file))
            })
            .map(|d| d.id)
            .collect()
    }

    pub fn check_assumption_invalidated(
        decisions: &[DecisionMemory],
        invalidated_assumptions: &[String],
    ) -> Vec<DecisionId> {
        decisions
            .iter()
            .filter(|d| d.state == DecisionState::Accepted)
            .filter(|d| {
                d.reasoning
                    .assumptions
                    .iter()
                    .any(|a| invalidated_assumptions.contains(&a.statement))
            })
            .map(|d| d.id)
            .collect()
    }
}
