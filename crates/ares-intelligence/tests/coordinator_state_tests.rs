use ares_intelligence::state::coordinator_state_machine::{
    CoordinatorState, CoordinatorStateMachine,
};

#[test]
fn test_coordinator_valid_full_lifecycle() {
    let mut sm = CoordinatorStateMachine::new();
    assert_eq!(sm.current_state(), CoordinatorState::Created);

    assert!(sm.transition_to(CoordinatorState::Analyzed).is_ok());
    assert_eq!(sm.current_state(), CoordinatorState::Analyzed);

    assert!(sm.transition_to(CoordinatorState::Selected).is_ok());
    assert!(sm.transition_to(CoordinatorState::Routed).is_ok());
    assert!(sm.transition_to(CoordinatorState::Evaluated).is_ok());
    assert!(sm.transition_to(CoordinatorState::Learned).is_ok());
    assert!(sm.transition_to(CoordinatorState::Synced).is_ok());

    assert_eq!(sm.current_state(), CoordinatorState::Synced);
}

#[test]
fn test_coordinator_invalid_transitions() {
    let mut sm = CoordinatorStateMachine::new();

    // Cannot skip Analyzed straight to Selected
    let res = sm.transition_to(CoordinatorState::Selected);
    assert!(res.is_err());
    assert_eq!(sm.current_state(), CoordinatorState::Created);

    // Cannot go backwards
    sm.transition_to(CoordinatorState::Analyzed).unwrap();
    let res = sm.transition_to(CoordinatorState::Created);
    assert!(res.is_err());
    assert_eq!(sm.current_state(), CoordinatorState::Analyzed);

    // Cannot jump to Learned
    let res = sm.transition_to(CoordinatorState::Learned);
    assert!(res.is_err());
}

#[test]
fn test_coordinator_failure_transition() {
    let mut sm = CoordinatorStateMachine::new();
    sm.transition_to(CoordinatorState::Analyzed).unwrap();
    sm.transition_to(CoordinatorState::Selected).unwrap();

    // Fail during routing
    assert!(sm.transition_to(CoordinatorState::Failed).is_ok());
    assert_eq!(sm.current_state(), CoordinatorState::Failed);

    // Cannot recover from failed (for now)
    let res = sm.transition_to(CoordinatorState::Routed);
    assert!(res.is_err());
}
