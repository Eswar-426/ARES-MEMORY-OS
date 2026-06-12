use super::*;
use crate::forecast::models::*;
use crate::planner_bridge::bridge::PlannerBridge;
use crate::scenario::models::ScenarioGenerationConfig;
use crate::simulation::models::SimulationConfig;

/// Regression: Empty goal title doesn't panic.
#[test]
fn empty_goal_title_no_panic() {
    let mut bridge = PlannerBridge::new();
    let state = make_world_state();
    let decision = bridge.evaluate_goal(
        "g1",
        "",
        &state,
        &[],
        &ScenarioGenerationConfig::default(),
        &SimulationConfig::default(),
        &RankingWeights::default(),
    );
    assert!(!decision.rankings.is_empty());
}

/// Regression: Very long goal title doesn't overflow.
#[test]
fn very_long_goal_title() {
    let mut bridge = PlannerBridge::new();
    let state = make_world_state();
    let long_title = "a".repeat(10000);
    let decision = bridge.evaluate_goal(
        "g1",
        &long_title,
        &state,
        &[],
        &ScenarioGenerationConfig::default(),
        &SimulationConfig::default(),
        &RankingWeights::default(),
    );
    assert!(!decision.rankings.is_empty());
}

/// Regression: Zero budget doesn't cause division by zero.
#[test]
fn zero_budget_no_div_by_zero() {
    let mut state = make_world_state();
    state.resources[0].available = 0.0;
    state.resources[0].capacity = 0.0;
    let mut bridge = PlannerBridge::new();
    let decision = bridge.evaluate_goal(
        "g1",
        "Build API",
        &state,
        &[],
        &ScenarioGenerationConfig::default(),
        &SimulationConfig::default(),
        &RankingWeights::default(),
    );
    assert!(!decision.rankings.is_empty());
    for r in &decision.rankings {
        assert!(!r.composite_score.is_nan());
    }
}

/// Regression: Single agent scenario works.
#[test]
fn single_agent_scenario() {
    let mut state = make_world_state();
    state.active_agents.truncate(1);
    let mut bridge = PlannerBridge::new();
    let decision = bridge.evaluate_goal(
        "g1",
        "Build API",
        &state,
        &[],
        &ScenarioGenerationConfig::default(),
        &SimulationConfig::default(),
        &RankingWeights::default(),
    );
    assert!(!decision.rankings.is_empty());
}

/// Regression: All constraints violated doesn't panic.
#[test]
fn all_constraints_violated() {
    let mut state = make_world_state();
    for c in &mut state.constraints {
        c.current = c.value * 2.0;
    }
    let mut bridge = PlannerBridge::new();
    let decision = bridge.evaluate_goal(
        "g1",
        "Build API",
        &state,
        &[],
        &ScenarioGenerationConfig::default(),
        &SimulationConfig::default(),
        &RankingWeights::default(),
    );
    assert!(!decision.rankings.is_empty());
}

/// Regression: Prediction with all success=false history.
#[test]
fn all_failure_history() {
    let mut predictor = crate::forecast::outcome_predictor::OutcomePredictor::new();
    let sim = crate::simulation::models::SimulationResult {
        id: ares_core::SimulationId::new(),
        scenario_id: ares_core::ScenarioId::new(),
        task_duration_secs: 3600.0,
        total_cost: 50.0,
        success_probability: 0.8,
        agent_utilization: 0.7,
        memory_usage_estimate: 100.0,
        risk_score: 0.2,
        simulated_at: chrono::Utc::now(),
    };
    let similar = vec![SimilarityMatch {
        mission: HistoricalMission {
            id: "f1".into(),
            title: "Failed Build".into(),
            keywords: vec!["build".into()],
            cost: 100.0,
            duration_secs: 7200.0,
            success: false,
            agent_count: 2,
            step_count: 5,
            completed_at: 1000,
        },
        similarity_score: 0.8,
        matching_keywords: vec!["build".into()],
    }];
    let prediction = predictor.predict("g1", &sim, &similar);
    assert!(prediction.success_probability >= 0.01);
    assert!(prediction.success_probability <= 0.99);
}

/// Regression: Max steps = 0 produces empty steps.
#[test]
fn max_steps_zero() {
    let gen = crate::scenario::generator::ScenarioGenerator::new();
    let state = make_world_state();
    let config = ScenarioGenerationConfig {
        max_steps: 0,
        ..Default::default()
    };
    let scenarios = gen.generate("g1", "Build API", &state, &config);
    for s in &scenarios {
        assert!(s.steps.is_empty());
    }
}

/// Regression: Forecast deviation with zero predicted cost.
#[test]
fn zero_predicted_cost_deviation() {
    let score = ForecastDeviation::calculate_deviation(0.0, 50.0, 3600.0, 3600.0, 0.8, true);
    assert!(!score.is_nan());
    assert!(score >= 0.0);
}

/// Regression: Empty constraint list doesn't affect results.
#[test]
fn empty_constraints() {
    let mut state = make_world_state();
    state.constraints.clear();
    let mut bridge = PlannerBridge::new();
    let decision = bridge.evaluate_goal(
        "g1",
        "Build API",
        &state,
        &[],
        &ScenarioGenerationConfig::default(),
        &SimulationConfig::default(),
        &RankingWeights::default(),
    );
    assert!(!decision.rankings.is_empty());
}

/// Regression: Counterfactual with parameter = 1.0 (extreme).
#[test]
fn extreme_counterfactual_parameter() {
    let cf_engine = crate::prediction::counterfactual::CounterfactualEngine::new();
    let sim = crate::simulation::models::SimulationResult {
        id: ares_core::SimulationId::new(),
        scenario_id: ares_core::ScenarioId::new(),
        task_duration_secs: 3600.0,
        total_cost: 50.0,
        success_probability: 0.8,
        agent_utilization: 0.7,
        memory_usage_estimate: 100.0,
        risk_score: 0.2,
        simulated_at: chrono::Utc::now(),
    };
    let cf = crate::prediction::models::Counterfactual {
        id: "cf_extreme".into(),
        counterfactual_type: crate::prediction::models::CounterfactualType::ProviderUnavailable,
        description: "Provider completely unavailable".into(),
        parameter: 1.0,
    };
    let result = cf_engine.evaluate(&cf, &sim);
    assert!(result.adjusted_success_probability >= 0.01);
    assert!(!result.impact_score.is_nan());
}
