mod counterfactual_tests;
mod explain_tests;
mod forecast_tests;
mod integration_tests;
mod persistence_tests;
mod planner_bridge_tests;
mod regression_tests;
mod risk_tests;
mod scenario_tests;
mod similarity_tests;
mod simulation_tests;
mod state_tests;
mod strategy_ranker_tests;

// ── Shared test helpers ──────────────────────────────────────────

use crate::scenario::models::*;
use crate::state::models::*;

pub fn make_world_state() -> WorldState {
    WorldState {
        id: ares_core::WorldStateId::new(),
        goals: vec![WorldGoal {
            id: "g1".into(),
            title: "Build App".into(),
            priority: "high".into(),
            status: "active".into(),
        }],
        resources: vec![
            WorldResource {
                name: "budget".into(),
                resource_type: ResourceType::Budget,
                available: 100.0,
                capacity: 100.0,
            },
            WorldResource {
                name: "compute".into(),
                resource_type: ResourceType::Compute,
                available: 80.0,
                capacity: 100.0,
            },
        ],
        active_agents: vec![
            WorldAgent {
                id: "a1".into(),
                name: "Agent Alpha".into(),
                role: "coder".into(),
                status: "ready".into(),
                success_rate: 0.85,
            },
            WorldAgent {
                id: "a2".into(),
                name: "Agent Beta".into(),
                role: "tester".into(),
                status: "ready".into(),
                success_rate: 0.90,
            },
        ],
        constraints: vec![WorldConstraint {
            name: "max_budget".into(),
            constraint_type: ConstraintType::MaxBudget,
            value: 100.0,
            current: 0.0,
            is_hard: true,
        }],
        snapshot_at: chrono::Utc::now(),
    }
}

pub fn make_world_state_tight_budget() -> WorldState {
    let mut state = make_world_state();
    state.resources[0].available = 10.0; // tight budget
    state.constraints[0].value = 10.0;
    state
}

pub fn make_world_state_no_agents() -> WorldState {
    let mut state = make_world_state();
    state.active_agents.clear();
    state
}

pub fn make_scenario_config() -> ScenarioGenerationConfig {
    ScenarioGenerationConfig::default()
}
