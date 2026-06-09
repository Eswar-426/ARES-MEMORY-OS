use crate::models::plan::PlanState;
use ares_core::AresError;

pub struct PlanStateMachine;

impl PlanStateMachine {
    /// Validates whether a transition from `current` to `next` is allowed.
    pub fn can_transition(current: &PlanState, next: &PlanState) -> bool {
        match (current, next) {
            // Forward happy path
            (PlanState::Draft, PlanState::Generated) => true,
            (PlanState::Generated, PlanState::Simulated) => true,
            (PlanState::Simulated, PlanState::Approved) => true,
            (PlanState::Approved, PlanState::Scheduled) => true,
            (PlanState::Scheduled, PlanState::Executing) => true,
            (PlanState::Executing, PlanState::Completed) => true,

            // Failure paths
            (PlanState::Executing, PlanState::Failed) => true,
            (PlanState::Failed, PlanState::Replanned) => true,

            // Cancelled can happen from almost any active state before completion
            (PlanState::Draft, PlanState::Cancelled) => true,
            (PlanState::Generated, PlanState::Cancelled) => true,
            (PlanState::Simulated, PlanState::Cancelled) => true,
            (PlanState::Approved, PlanState::Cancelled) => true,
            (PlanState::Scheduled, PlanState::Cancelled) => true,
            (PlanState::Executing, PlanState::Cancelled) => true,

            _ => false,
        }
    }

    /// Attempts a transition and returns an error if invalid.
    pub fn transition(current: &PlanState, next: PlanState) -> Result<PlanState, AresError> {
        if Self::can_transition(current, &next) {
            Ok(next)
        } else {
            Err(AresError::validation(format!(
                "Invalid plan state transition from {:?} to {:?}",
                current, next
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_forward_transitions() {
        assert!(PlanStateMachine::can_transition(
            &PlanState::Generated,
            &PlanState::Simulated
        ));
        assert!(PlanStateMachine::can_transition(
            &PlanState::Simulated,
            &PlanState::Approved
        ));
        assert!(PlanStateMachine::can_transition(
            &PlanState::Approved,
            &PlanState::Scheduled
        ));
        assert!(PlanStateMachine::can_transition(
            &PlanState::Scheduled,
            &PlanState::Executing
        ));
        assert!(PlanStateMachine::can_transition(
            &PlanState::Executing,
            &PlanState::Completed
        ));
    }

    #[test]
    fn test_failure_and_replanning_transitions() {
        assert!(PlanStateMachine::can_transition(
            &PlanState::Executing,
            &PlanState::Failed
        ));
        assert!(PlanStateMachine::can_transition(
            &PlanState::Failed,
            &PlanState::Replanned
        ));
    }

    #[test]
    fn test_cancellation_paths() {
        assert!(PlanStateMachine::can_transition(
            &PlanState::Draft,
            &PlanState::Cancelled
        ));
        assert!(PlanStateMachine::can_transition(
            &PlanState::Executing,
            &PlanState::Cancelled
        ));
    }

    #[test]
    fn test_invalid_transitions() {
        // Cannot go backwards
        assert!(!PlanStateMachine::can_transition(
            &PlanState::Executing,
            &PlanState::Scheduled
        ));

        // Cannot execute from completed
        assert!(!PlanStateMachine::can_transition(
            &PlanState::Completed,
            &PlanState::Executing
        ));

        // Cannot approve from cancelled
        assert!(!PlanStateMachine::can_transition(
            &PlanState::Cancelled,
            &PlanState::Approved
        ));
    }

    #[test]
    fn test_transition_function() {
        let result = PlanStateMachine::transition(&PlanState::Simulated, PlanState::Approved);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PlanState::Approved);

        let err_result = PlanStateMachine::transition(&PlanState::Completed, PlanState::Executing);
        assert!(err_result.is_err());
    }
}
