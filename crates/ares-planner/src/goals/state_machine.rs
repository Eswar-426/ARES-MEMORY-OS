use crate::models::goal::GoalState;
use ares_core::AresError;

pub struct GoalStateMachine;

impl GoalStateMachine {
    /// Validates whether a transition from `current` to `next` is allowed.
    pub fn can_transition(current: &GoalState, next: &GoalState) -> bool {
        match (current, next) {
            // Forward happy path
            (GoalState::Draft, GoalState::Ready) => true,
            (GoalState::Ready, GoalState::Planning) => true,
            (GoalState::Planning, GoalState::Planned) => true,
            (GoalState::Planned, GoalState::Executing) => true,
            (GoalState::Executing, GoalState::Completed) => true,

            // Failure paths
            (GoalState::Planning, GoalState::PlanningFailed) => true,
            (GoalState::Executing, GoalState::ExecutionFailed) => true,

            // Cancelled can happen from almost any active state
            (GoalState::Draft, GoalState::Cancelled) => true,
            (GoalState::Ready, GoalState::Cancelled) => true,
            (GoalState::Planning, GoalState::Cancelled) => true,
            (GoalState::Planned, GoalState::Cancelled) => true,
            (GoalState::Executing, GoalState::Cancelled) => true,
            (GoalState::Blocked, GoalState::Cancelled) => true,

            // Blocked can happen when waiting for user input or external resources
            (GoalState::Planning, GoalState::Blocked) => true,
            (GoalState::Planned, GoalState::Blocked) => true,
            (GoalState::Executing, GoalState::Blocked) => true,

            // Recovery paths
            (GoalState::PlanningFailed, GoalState::Planning) => true,
            (GoalState::ExecutionFailed, GoalState::Planning) => true, // Replanning
            (GoalState::Blocked, GoalState::Ready) => true,
            (GoalState::Blocked, GoalState::Planning) => true,
            (GoalState::Blocked, GoalState::Planned) => true,
            (GoalState::Blocked, GoalState::Executing) => true,

            _ => false,
        }
    }

    /// Attempts a transition and returns an error if invalid.
    pub fn transition(current: &GoalState, next: GoalState) -> Result<GoalState, AresError> {
        if Self::can_transition(current, &next) {
            Ok(next)
        } else {
            Err(AresError::validation(format!(
                "Invalid goal state transition from {:?} to {:?}",
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
        assert!(GoalStateMachine::can_transition(
            &GoalState::Draft,
            &GoalState::Ready
        ));
        assert!(GoalStateMachine::can_transition(
            &GoalState::Ready,
            &GoalState::Planning
        ));
        assert!(GoalStateMachine::can_transition(
            &GoalState::Planning,
            &GoalState::Planned
        ));
        assert!(GoalStateMachine::can_transition(
            &GoalState::Planned,
            &GoalState::Executing
        ));
        assert!(GoalStateMachine::can_transition(
            &GoalState::Executing,
            &GoalState::Completed
        ));
    }

    #[test]
    fn test_failure_paths() {
        assert!(GoalStateMachine::can_transition(
            &GoalState::Planning,
            &GoalState::PlanningFailed
        ));
        assert!(GoalStateMachine::can_transition(
            &GoalState::Executing,
            &GoalState::ExecutionFailed
        ));
    }

    #[test]
    fn test_administrative_and_recovery_paths() {
        // Administrative
        assert!(GoalStateMachine::can_transition(
            &GoalState::Ready,
            &GoalState::Cancelled
        ));
        assert!(GoalStateMachine::can_transition(
            &GoalState::Executing,
            &GoalState::Cancelled
        ));
        assert!(GoalStateMachine::can_transition(
            &GoalState::Planning,
            &GoalState::Blocked
        ));
        assert!(GoalStateMachine::can_transition(
            &GoalState::Blocked,
            &GoalState::Cancelled
        ));

        // Recovery
        assert!(GoalStateMachine::can_transition(
            &GoalState::Blocked,
            &GoalState::Ready
        ));
        assert!(GoalStateMachine::can_transition(
            &GoalState::Blocked,
            &GoalState::Planning
        ));
    }

    #[test]
    fn test_invalid_transitions() {
        assert!(!GoalStateMachine::can_transition(
            &GoalState::Completed,
            &GoalState::Planning
        ));
        assert!(!GoalStateMachine::can_transition(
            &GoalState::Cancelled,
            &GoalState::Ready
        ));
        assert!(!GoalStateMachine::can_transition(
            &GoalState::Draft,
            &GoalState::Executing
        ));
    }

    #[test]
    fn test_transition_function() {
        assert!(GoalStateMachine::transition(&GoalState::Draft, GoalState::Ready).is_ok());
        let err = GoalStateMachine::transition(&GoalState::Completed, GoalState::Planning);
        assert!(err.is_err());
    }
}
