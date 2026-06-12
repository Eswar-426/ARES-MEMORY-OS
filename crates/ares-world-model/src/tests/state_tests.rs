use super::*;
use crate::state::engine::WorldStateEngine;
use crate::state::models::*;

#[test]
fn world_state_new_creates_empty() {
    let state = WorldState::new();
    assert!(state.goals.is_empty());
    assert!(state.resources.is_empty());
    assert!(state.active_agents.is_empty());
    assert!(state.constraints.is_empty());
}

#[test]
fn world_state_default_equals_new() {
    let s1 = WorldState::new();
    let s2 = WorldState::default();
    assert_eq!(s1.goals.len(), s2.goals.len());
}

#[test]
fn total_budget_sums_budget_resources() {
    let state = make_world_state();
    assert!((state.total_budget() - 100.0).abs() < f64::EPSILON);
}

#[test]
fn total_budget_zero_when_no_budget_resources() {
    let mut state = WorldState::new();
    state.resources.push(WorldResource {
        name: "compute".into(),
        resource_type: ResourceType::Compute,
        available: 50.0,
        capacity: 100.0,
    });
    assert!((state.total_budget() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn available_agent_count_filters_ready() {
    let state = make_world_state();
    assert_eq!(state.available_agent_count(), 2);
}

#[test]
fn available_agent_count_zero_when_all_busy() {
    let mut state = make_world_state();
    for a in &mut state.active_agents {
        a.status = "busy".into();
    }
    assert_eq!(state.available_agent_count(), 0);
}

#[test]
fn average_agent_success_rate_computed() {
    let state = make_world_state();
    let avg = state.average_agent_success_rate();
    assert!((avg - 0.875).abs() < 0.01);
}

#[test]
fn average_agent_success_rate_zero_when_empty() {
    let state = WorldState::new();
    assert!((state.average_agent_success_rate() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn has_violated_constraints_false_when_ok() {
    let state = make_world_state();
    assert!(!state.has_violated_constraints());
}

#[test]
fn has_violated_constraints_true_when_exceeded() {
    let mut state = make_world_state();
    state.constraints[0].current = 200.0; // exceeds max_budget of 100
    assert!(state.has_violated_constraints());
}

#[test]
fn resource_utilization_correct() {
    let r = WorldResource {
        name: "cpu".into(),
        resource_type: ResourceType::Compute,
        available: 30.0,
        capacity: 100.0,
    };
    assert!((r.utilization() - 0.7).abs() < 0.01);
}

#[test]
fn resource_utilization_zero_capacity() {
    let r = WorldResource {
        name: "x".into(),
        resource_type: ResourceType::Custom("x".into()),
        available: 0.0,
        capacity: 0.0,
    };
    assert!((r.utilization() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn resource_utilization_clamped() {
    let r = WorldResource {
        name: "x".into(),
        resource_type: ResourceType::Compute,
        available: -10.0,
        capacity: 100.0,
    };
    assert!(r.utilization() <= 1.0);
    assert!(r.utilization() >= 0.0);
}

#[test]
fn constraint_violated_max_budget() {
    let c = WorldConstraint {
        name: "b".into(),
        constraint_type: ConstraintType::MaxBudget,
        value: 100.0,
        current: 150.0,
        is_hard: true,
    };
    assert!(c.is_violated());
}

#[test]
fn constraint_not_violated_max_budget() {
    let c = WorldConstraint {
        name: "b".into(),
        constraint_type: ConstraintType::MaxBudget,
        value: 100.0,
        current: 50.0,
        is_hard: true,
    };
    assert!(!c.is_violated());
}

#[test]
fn constraint_violated_min_quality() {
    let c = WorldConstraint {
        name: "q".into(),
        constraint_type: ConstraintType::MinQuality,
        value: 0.8,
        current: 0.5,
        is_hard: true,
    };
    assert!(c.is_violated());
}

#[test]
fn constraint_slack_positive_when_safe() {
    let c = WorldConstraint {
        name: "b".into(),
        constraint_type: ConstraintType::MaxBudget,
        value: 100.0,
        current: 60.0,
        is_hard: true,
    };
    assert!((c.slack() - 40.0).abs() < f64::EPSILON);
}

#[test]
fn constraint_slack_negative_when_violated() {
    let c = WorldConstraint {
        name: "b".into(),
        constraint_type: ConstraintType::MaxBudget,
        value: 100.0,
        current: 120.0,
        is_hard: true,
    };
    assert!(c.slack() < 0.0);
}

#[test]
fn world_state_serialization_roundtrip() {
    let engine = WorldStateEngine::new();
    let state = make_world_state();
    let json = engine.to_json(&state).unwrap();
    let back = engine.from_json(&json).unwrap();
    assert_eq!(back.goals.len(), state.goals.len());
    assert_eq!(back.resources.len(), state.resources.len());
}

#[test]
fn engine_diff_no_changes() {
    let engine = WorldStateEngine::new();
    let state = make_world_state();
    let diff = engine.diff_states(&state, &state);
    assert_eq!(diff.goals_added, 0);
    assert_eq!(diff.goals_removed, 0);
    assert!((diff.budget_delta - 0.0).abs() < f64::EPSILON);
}

#[test]
fn engine_diff_detects_added_goal() {
    let engine = WorldStateEngine::new();
    let old = make_world_state();
    let mut new = old.clone();
    new.goals.push(WorldGoal {
        id: "g2".into(),
        title: "New".into(),
        priority: "low".into(),
        status: "draft".into(),
    });
    let diff = engine.diff_states(&old, &new);
    assert_eq!(diff.goals_added, 1);
}

#[test]
fn engine_diff_detects_removed_goal() {
    let engine = WorldStateEngine::new();
    let old = make_world_state();
    let mut new = old.clone();
    new.goals.clear();
    let diff = engine.diff_states(&old, &new);
    assert_eq!(diff.goals_removed, 1);
}

#[test]
fn engine_validate_catches_negative_availability() {
    let engine = WorldStateEngine::new();
    let mut state = make_world_state();
    state.resources[0].available = -10.0;
    let issues = engine.validate(&state);
    assert!(!issues.is_empty());
    assert!(issues[0].contains("negative availability"));
}

#[test]
fn engine_validate_catches_invalid_success_rate() {
    let engine = WorldStateEngine::new();
    let mut state = make_world_state();
    state.active_agents[0].success_rate = 1.5;
    let issues = engine.validate(&state);
    assert!(!issues.is_empty());
}

#[test]
fn engine_validate_catches_violated_hard_constraint() {
    let engine = WorldStateEngine::new();
    let mut state = make_world_state();
    state.constraints[0].current = 200.0;
    let issues = engine.validate(&state);
    assert!(issues.iter().any(|i| i.contains("violated")));
}

#[test]
fn engine_validate_passes_clean_state() {
    let engine = WorldStateEngine::new();
    let state = make_world_state();
    let issues = engine.validate(&state);
    assert!(issues.is_empty());
}
