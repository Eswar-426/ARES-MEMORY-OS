use ares_core::id::{GoalId, PlanId};
use ares_core::AresError;

pub struct FeedbackService;

impl FeedbackService {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    /// Records feedback from a completed or failed execution.
    pub fn record_execution_feedback(
        &self,
        _plan_id: &PlanId,
        _goal_id: &GoalId,
        success: bool,
        duration_sec: f64,
        cost: f64,
        notes: Option<String>,
    ) -> Result<(), AresError> {
        // In the future:
        // 1. Compare actual duration vs estimated duration
        // 2. Compare actual cost vs estimated cost
        // 3. If failed, log failure context
        // 4. Send this learning back to the Knowledge Graph (ares-knowledge)

        let _feedback_record = (success, duration_sec, cost, notes);

        // TODO: Persist to feedback repository
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_execution_feedback() {
        let service = FeedbackService::new();
        let plan_id = PlanId::new();
        let goal_id = GoalId::new();

        let result = service.record_execution_feedback(
            &plan_id,
            &goal_id,
            true,
            30.0,
            5.0,
            Some("Worked great".to_string()),
        );

        assert!(result.is_ok());
    }
}
