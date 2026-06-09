use crate::events::publisher::PlannerEventPublisher;
use ares_core::id::{GoalId, PlanId};
use ares_core::types::workflow::{WorkflowEventType, WorkflowStatus};
use ares_core::AresError;
use std::sync::Arc;

pub struct PlanExecutionTracker {
    publisher: Arc<PlannerEventPublisher>,
}

impl PlanExecutionTracker {
    pub fn new(publisher: Arc<PlannerEventPublisher>) -> Self {
        Self { publisher }
    }

    /// Handles events emitted by the orchestrator and translates them back into Planner lifecycle states.
    pub fn handle_orchestrator_event(
        &self,
        event_type: &WorkflowEventType,
        plan_id: &PlanId,
        _goal_id: &GoalId,
        _status: &WorkflowStatus,
    ) -> Result<(), AresError> {
        match event_type {
            WorkflowEventType::WorkflowStarted => {
                // E.g., Move goal to GoalState::Executing
                // Publisher emits PlanStarted
                self.publisher.publish_plan_started(
                    None,
                    crate::events::models::PlanStartedPayload {
                        plan_id: plan_id.clone(),
                    },
                )?;
            }
            WorkflowEventType::WorkflowCompleted => {
                // E.g., Move goal to GoalState::Completed
                self.publisher.publish_plan_completed(
                    None,
                    crate::events::models::PlanCompletedPayload {
                        plan_id: plan_id.clone(),
                    },
                )?;
            }
            WorkflowEventType::WorkflowFailed | WorkflowEventType::WorkflowTimedOut => {
                // Trigger replanning!
                self.publisher.publish_replanning_triggered(
                    None,
                    crate::events::models::ReplanningTriggeredPayload {
                        plan_id: plan_id.clone(),
                        reason: "Orchestrator reported workflow failure or timeout".into(),
                    },
                )?;
            }
            _ => {
                // Ignore step-level events for now.
            }
        }

        Ok(())
    }
}
