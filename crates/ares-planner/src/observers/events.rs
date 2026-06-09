use crate::execution::tracker::PlanExecutionTracker;
use ares_core::id::{GoalId, PlanId};
use ares_core::types::workflow::{WorkflowEventType, WorkflowStatus};
use ares_core::AresError;
use std::sync::Arc;

pub struct ExecutionObserver {
    tracker: Arc<PlanExecutionTracker>,
}

impl ExecutionObserver {
    pub fn new(tracker: Arc<PlanExecutionTracker>) -> Self {
        Self { tracker }
    }

    pub fn on_orchestrator_event(
        &self,
        event_type: &WorkflowEventType,
        plan_id: &PlanId,
        goal_id: &GoalId,
        status: &WorkflowStatus,
    ) -> Result<(), AresError> {
        self.tracker
            .handle_orchestrator_event(event_type, plan_id, goal_id, status)
    }
}
