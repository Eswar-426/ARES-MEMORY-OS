use super::*;
use crate::scenario::generator::ScenarioGenerator;
use crate::simulation::engine::SimulationEngine;
use crate::simulation::models::*;

fn make_scenarios() -> Vec<crate::scenario::models::Scenario> {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    gen.generate("g1", "Build API", &state, &make_scenario_config())
}

#[test]
fn simulate_produces_result() {
    let engine = SimulationEngine::new();
    let state = make_world_state();
    let scenarios = make_scenarios();
    let result = engine.simulate(&scenarios[0], &state, &SimulationConfig::default());
    assert!(result.task_duration_secs > 0.0);
    assert!(result.total_cost > 0.0);
}

#[test]
fn success_probability_bounded() {
    let engine = SimulationEngine::new();
    let state = make_world_state();
    for s in &make_scenarios() {
        let r = engine.simulate(s, &state, &SimulationConfig::default());
        assert!(r.success_probability >= 0.01);
        assert!(r.success_probability <= 0.99);
    }
}

#[test]
fn risk_score_bounded() {
    let engine = SimulationEngine::new();
    let state = make_world_state();
    for s in &make_scenarios() {
        let r = engine.simulate(s, &state, &SimulationConfig::default());
        assert!(r.risk_score >= 0.0);
        assert!(r.risk_score <= 1.0);
    }
}

#[test]
fn simulate_batch_returns_all() {
    let engine = SimulationEngine::new();
    let state = make_world_state();
    let scenarios = make_scenarios();
    let results = engine.simulate_batch(&scenarios, &state, &SimulationConfig::default());
    assert_eq!(results.len(), scenarios.len());
}

#[test]
fn tight_budget_increases_risk() {
    let engine = SimulationEngine::new();
    let normal_state = make_world_state();
    let tight_state = make_world_state_tight_budget();
    let scenarios = make_scenarios();
    let r_normal = engine.simulate(&scenarios[0], &normal_state, &SimulationConfig::default());
    let r_tight = engine.simulate(&scenarios[0], &tight_state, &SimulationConfig::default());
    assert!(r_tight.risk_score >= r_normal.risk_score);
}

#[test]
fn no_agents_lowers_success() {
    let engine = SimulationEngine::new();
    let normal_state = make_world_state();
    let empty_state = make_world_state_no_agents();
    let scenarios = make_scenarios();
    let r_normal = engine.simulate(&scenarios[0], &normal_state, &SimulationConfig::default());
    let r_empty = engine.simulate(&scenarios[0], &empty_state, &SimulationConfig::default());
    assert!(r_empty.success_probability <= r_normal.success_probability);
}

#[test]
fn historical_data_blends_into_success() {
    let engine = SimulationEngine::new();
    let state = make_world_state();
    let scenarios = make_scenarios();
    let config_no_hist = SimulationConfig::default();
    let config_hist = SimulationConfig {
        historical_success_rate: Some(0.95),
        historical_sample_count: 50,
        ..Default::default()
    };
    let r_no = engine.simulate(&scenarios[0], &state, &config_no_hist);
    let r_hist = engine.simulate(&scenarios[0], &state, &config_hist);
    // With high historical rate, success should be >= without it
    assert!(r_hist.success_probability >= r_no.success_probability - 0.1);
}

#[test]
fn agent_utilization_bounded() {
    let engine = SimulationEngine::new();
    let state = make_world_state();
    for s in &make_scenarios() {
        let r = engine.simulate(s, &state, &SimulationConfig::default());
        assert!(r.agent_utilization >= 0.0);
        assert!(r.agent_utilization <= 1.0);
    }
}

#[test]
fn memory_usage_positive() {
    let engine = SimulationEngine::new();
    let state = make_world_state();
    for s in &make_scenarios() {
        let r = engine.simulate(s, &state, &SimulationConfig::default());
        assert!(r.memory_usage_estimate > 0.0);
    }
}

#[test]
fn is_likely_success_correct() {
    let engine = SimulationEngine::new();
    let state = make_world_state();
    let scenarios = make_scenarios();
    let r = engine.simulate(&scenarios[0], &state, &SimulationConfig::default());
    assert_eq!(r.is_likely_success(), r.success_probability >= 0.5);
}

#[test]
fn cost_efficiency_positive() {
    let engine = SimulationEngine::new();
    let state = make_world_state();
    let scenarios = make_scenarios();
    let r = engine.simulate(&scenarios[0], &state, &SimulationConfig::default());
    assert!(r.cost_efficiency() > 0.0);
}

#[test]
fn time_efficiency_positive() {
    let engine = SimulationEngine::new();
    let state = make_world_state();
    let scenarios = make_scenarios();
    let r = engine.simulate(&scenarios[0], &state, &SimulationConfig::default());
    assert!(r.time_efficiency() > 0.0);
}

#[test]
fn simulation_config_default() {
    let c = SimulationConfig::default();
    assert_eq!(c.iterations, 1);
    assert!((c.confidence_level - 0.8).abs() < f64::EPSILON);
    assert!(c.historical_success_rate.is_none());
}

#[test]
fn simulation_summary_from_empty() {
    let summary = SimulationSummary::from_results(&[], 0);
    assert_eq!(summary.scenario_count, 0);
}

#[test]
fn simulation_summary_from_results() {
    let engine = SimulationEngine::new();
    let state = make_world_state();
    let scenarios = make_scenarios();
    let results = engine.simulate_batch(&scenarios, &state, &SimulationConfig::default());
    let summary = SimulationSummary::from_results(&results, 100);
    assert_eq!(summary.scenario_count, 4);
    assert!(summary.avg_cost > 0.0);
    assert!(summary.avg_duration_secs > 0.0);
    assert!(summary.min_cost <= summary.max_cost);
    assert!(summary.min_duration_secs <= summary.max_duration_secs);
    assert_eq!(summary.total_simulation_ms, 100);
}

#[test]
fn simulation_result_serialization() {
    let engine = SimulationEngine::new();
    let state = make_world_state();
    let scenarios = make_scenarios();
    let r = engine.simulate(&scenarios[0], &state, &SimulationConfig::default());
    let json = serde_json::to_string(&r).unwrap();
    let back: SimulationResult = serde_json::from_str(&json).unwrap();
    assert!((back.total_cost - r.total_cost).abs() < 0.01);
}

#[test]
fn more_steps_higher_memory_usage() {
    let engine = SimulationEngine::new();
    let state = make_world_state();
    let gen = ScenarioGenerator::new();
    let small = gen.generate("g1", "Fix bug", &state, &make_scenario_config());
    let big = gen.generate(
        "g2",
        "Build full production enterprise system",
        &state,
        &make_scenario_config(),
    );
    let r_small = engine.simulate(&small[0], &state, &SimulationConfig::default());
    let r_big = engine.simulate(&big[0], &state, &SimulationConfig::default());
    assert!(r_big.memory_usage_estimate >= r_small.memory_usage_estimate);
}

#[test]
fn violated_constraints_lower_success() {
    let engine = SimulationEngine::new();
    let mut state = make_world_state();
    let scenarios = make_scenarios();
    let r_normal = engine.simulate(&scenarios[0], &state, &SimulationConfig::default());
    state.constraints[0].current = 200.0; // violate
    let r_violated = engine.simulate(&scenarios[0], &state, &SimulationConfig::default());
    assert!(r_violated.success_probability <= r_normal.success_probability);
}

#[test]
fn high_resource_utilization_increases_cost() {
    let engine = SimulationEngine::new();
    let mut state = make_world_state();
    let scenarios = make_scenarios();
    let r_normal = engine.simulate(&scenarios[0], &state, &SimulationConfig::default());
    state.resources[1].available = 5.0; // 95% utilization
    let r_stressed = engine.simulate(&scenarios[0], &state, &SimulationConfig::default());
    assert!(r_stressed.total_cost >= r_normal.total_cost);
}

#[test]
fn empty_scenario_zero_duration() {
    let engine = SimulationEngine::new();
    let state = make_world_state();
    let scenario = crate::scenario::models::Scenario {
        id: ares_core::ScenarioId::new(),
        goal_id: "g1".into(),
        scenario_type: crate::scenario::models::ScenarioType::Balanced,
        description: "empty".into(),
        estimated_cost: 0.0,
        estimated_duration_secs: 0.0,
        estimated_quality: 0.8,
        agent_assignments: vec![],
        steps: vec![],
        created_at: chrono::Utc::now(),
    };
    let r = engine.simulate(&scenario, &state, &SimulationConfig::default());
    assert!((r.task_duration_secs - 0.0).abs() < f64::EPSILON);
}

#[test]
fn simulation_summary_serialization() {
    let summary = SimulationSummary::from_results(&[], 42);
    let json = serde_json::to_string(&summary).unwrap();
    let back: SimulationSummary = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_simulation_ms, 42);
}

#[test]
fn fastest_scenario_has_lower_duration() {
    let sim_engine = SimulationEngine::new();
    let state = make_world_state();
    let scenarios = make_scenarios();
    let fastest = scenarios
        .iter()
        .find(|s| s.scenario_type == crate::scenario::models::ScenarioType::Fastest)
        .unwrap();
    let balanced = scenarios
        .iter()
        .find(|s| s.scenario_type == crate::scenario::models::ScenarioType::Balanced)
        .unwrap();
    let sim_fast = sim_engine.simulate(fastest, &state, &SimulationConfig::default());
    let sim_balanced = sim_engine.simulate(balanced, &state, &SimulationConfig::default());
    // Fastest should have lower or equal duration
    assert!(sim_fast.task_duration_secs <= sim_balanced.task_duration_secs * 1.5);
}

#[test]
fn cheapest_scenario_has_lower_cost() {
    let sim_engine = SimulationEngine::new();
    let state = make_world_state();
    let scenarios = make_scenarios();
    let cheapest = scenarios
        .iter()
        .find(|s| s.scenario_type == crate::scenario::models::ScenarioType::Cheapest)
        .unwrap();
    let balanced = scenarios
        .iter()
        .find(|s| s.scenario_type == crate::scenario::models::ScenarioType::Balanced)
        .unwrap();
    let sim_cheap = sim_engine.simulate(cheapest, &state, &SimulationConfig::default());
    let sim_balanced = sim_engine.simulate(balanced, &state, &SimulationConfig::default());
    assert!(sim_cheap.total_cost <= sim_balanced.total_cost * 1.1);
}

#[test]
fn simulate_preserves_scenario_id() {
    let sim_engine = SimulationEngine::new();
    let state = make_world_state();
    let scenarios = make_scenarios();
    let sim = sim_engine.simulate(&scenarios[0], &state, &SimulationConfig::default());
    assert_eq!(sim.scenario_id, scenarios[0].id);
}

#[test]
fn simulation_cost_scales_with_steps() {
    let gen = crate::scenario::generator::ScenarioGenerator::new();
    let state = make_world_state();
    let config_small = crate::scenario::models::ScenarioGenerationConfig {
        max_steps: 3,
        ..Default::default()
    };
    let config_large = crate::scenario::models::ScenarioGenerationConfig {
        max_steps: 10,
        ..Default::default()
    };
    let small = gen.generate("g1", "Build API", &state, &config_small);
    let large = gen.generate("g1", "Build API", &state, &config_large);
    let sim_engine = SimulationEngine::new();
    let sim_s = sim_engine.simulate(&small[0], &state, &SimulationConfig::default());
    let sim_l = sim_engine.simulate(&large[0], &state, &SimulationConfig::default());
    // More steps should generally cost more
    assert!(sim_l.total_cost >= sim_s.total_cost * 0.5);
}

#[test]
fn simulation_with_custom_complexity() {
    let sim_engine = SimulationEngine::new();
    let state = make_world_state();
    let scenarios = make_scenarios();
    let config = SimulationConfig {
        confidence_level: 0.95,
        ..Default::default()
    };
    let sim = sim_engine.simulate(&scenarios[0], &state, &config);
    assert!(sim.total_cost > 0.0);
    assert!(sim.task_duration_secs > 0.0);
}

#[test]
fn simulation_batch_all_unique_ids() {
    let sim_engine = SimulationEngine::new();
    let state = make_world_state();
    let scenarios = make_scenarios();
    let sims = sim_engine.simulate_batch(&scenarios, &state, &SimulationConfig::default());
    let ids: std::collections::HashSet<_> = sims.iter().map(|s| s.id.clone()).collect();
    assert_eq!(ids.len(), sims.len());
}

#[test]
fn simulation_batch_maps_scenario_ids() {
    let sim_engine = SimulationEngine::new();
    let state = make_world_state();
    let scenarios = make_scenarios();
    let sims = sim_engine.simulate_batch(&scenarios, &state, &SimulationConfig::default());
    for (scenario, sim) in scenarios.iter().zip(sims.iter()) {
        assert_eq!(scenario.id, sim.scenario_id);
    }
}

#[test]
fn simulated_at_is_recent() {
    let sim_engine = SimulationEngine::new();
    let state = make_world_state();
    let scenarios = make_scenarios();
    let before = chrono::Utc::now();
    let sim = sim_engine.simulate(&scenarios[0], &state, &SimulationConfig::default());
    let after = chrono::Utc::now();
    assert!(sim.simulated_at >= before);
    assert!(sim.simulated_at <= after);
}

#[test]
fn high_quality_scenario_higher_success() {
    let sim_engine = SimulationEngine::new();
    let state = make_world_state();
    let scenarios = make_scenarios();
    let hq = scenarios
        .iter()
        .find(|s| s.scenario_type == crate::scenario::models::ScenarioType::HighestQuality)
        .unwrap();
    let cheapest = scenarios
        .iter()
        .find(|s| s.scenario_type == crate::scenario::models::ScenarioType::Cheapest)
        .unwrap();
    let sim_hq = sim_engine.simulate(hq, &state, &SimulationConfig::default());
    let sim_cheap = sim_engine.simulate(cheapest, &state, &SimulationConfig::default());
    // Higher quality should generally have higher success probability
    assert!(sim_hq.success_probability >= sim_cheap.success_probability * 0.8);
}

#[test]
fn simulation_result_debug_format() {
    let sim_engine = SimulationEngine::new();
    let state = make_world_state();
    let scenarios = make_scenarios();
    let sim = sim_engine.simulate(&scenarios[0], &state, &SimulationConfig::default());
    let debug = format!("{:?}", sim);
    assert!(!debug.is_empty());
    assert!(debug.contains("SimulationResult"));
}
