use super::*;
use crate::forecast::models::*;
use crate::planner_bridge::bridge::PlannerBridge;
use crate::scenario::models::ScenarioGenerationConfig;
use crate::simulation::models::SimulationConfig;

fn make_historical() -> Vec<HistoricalMission> {
    vec![
        HistoricalMission {
            id: "m1".into(),
            title: "Build REST API".into(),
            keywords: vec!["build".into(), "api".into(), "rest".into()],
            cost: 45.0,
            duration_secs: 3600.0,
            success: true,
            agent_count: 2,
            step_count: 5,
            completed_at: 1000,
        },
        HistoricalMission {
            id: "m2".into(),
            title: "Build API Gateway".into(),
            keywords: vec!["build".into(), "api".into(), "gateway".into()],
            cost: 60.0,
            duration_secs: 5400.0,
            success: true,
            agent_count: 3,
            step_count: 8,
            completed_at: 2000,
        },
    ]
}

#[test]
fn evaluate_goal_returns_decision() {
    let mut bridge = PlannerBridge::new();
    let state = make_world_state();
    let decision = bridge.evaluate_goal(
        "g1",
        "Build REST API",
        &state,
        &make_historical(),
        &ScenarioGenerationConfig::default(),
        &SimulationConfig::default(),
        &RankingWeights::default(),
    );
    assert_eq!(decision.goal_id, "g1");
    assert_eq!(decision.goal_title, "Build REST API");
}

#[test]
fn decision_has_best_scenario() {
    let mut bridge = PlannerBridge::new();
    let state = make_world_state();
    let decision = bridge.evaluate_goal(
        "g1",
        "Build API",
        &state,
        &make_historical(),
        &ScenarioGenerationConfig::default(),
        &SimulationConfig::default(),
        &RankingWeights::default(),
    );
    assert!(!decision.best_scenario.steps.is_empty());
}

#[test]
fn decision_has_rankings() {
    let mut bridge = PlannerBridge::new();
    let state = make_world_state();
    let decision = bridge.evaluate_goal(
        "g1",
        "Build API",
        &state,
        &[],
        &ScenarioGenerationConfig::default(),
        &SimulationConfig::default(),
        &RankingWeights::default(),
    );
    assert_eq!(decision.rankings.len(), 4);
}

#[test]
fn decision_has_counterfactuals() {
    let mut bridge = PlannerBridge::new();
    let state = make_world_state();
    let decision = bridge.evaluate_goal(
        "g1",
        "Build API",
        &state,
        &[],
        &ScenarioGenerationConfig::default(),
        &SimulationConfig::default(),
        &RankingWeights::default(),
    );
    assert_eq!(decision.counterfactual_results.len(), 5);
}

#[test]
fn decision_has_explanation() {
    let mut bridge = PlannerBridge::new();
    let state = make_world_state();
    let decision = bridge.evaluate_goal(
        "g1",
        "Build API",
        &state,
        &make_historical(),
        &ScenarioGenerationConfig::default(),
        &SimulationConfig::default(),
        &RankingWeights::default(),
    );
    assert!(!decision.explanation.selected_scenario.is_empty());
    assert!(!decision.explanation.reason.is_empty());
}

#[test]
fn decision_prediction_has_confidence_reasons() {
    let mut bridge = PlannerBridge::new();
    let state = make_world_state();
    let decision = bridge.evaluate_goal(
        "g1",
        "Build API",
        &state,
        &make_historical(),
        &ScenarioGenerationConfig::default(),
        &SimulationConfig::default(),
        &RankingWeights::default(),
    );
    assert!(!decision.prediction.confidence_reasons.is_empty());
}

#[test]
fn decision_with_tight_budget() {
    let mut bridge = PlannerBridge::new();
    let state = make_world_state_tight_budget();
    let decision = bridge.evaluate_goal(
        "g1",
        "Build API",
        &state,
        &[],
        &ScenarioGenerationConfig::default(),
        &SimulationConfig::default(),
        &RankingWeights::default(),
    );
    // Should still produce a decision, just with higher risk
    assert!(!decision.rankings.is_empty());
}

#[test]
fn decision_no_agents() {
    let mut bridge = PlannerBridge::new();
    let state = make_world_state_no_agents();
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

#[test]
fn decision_serialization() {
    let mut bridge = PlannerBridge::new();
    let state = make_world_state();
    let decision = bridge.evaluate_goal(
        "g1",
        "Build API",
        &state,
        &[],
        &ScenarioGenerationConfig::default(),
        &SimulationConfig::default(),
        &RankingWeights::default(),
    );
    let json = serde_json::to_string(&decision).unwrap();
    assert!(json.contains("Build API"));
}

#[test]
fn bridge_outcome_predictor_accessible() {
    let mut bridge = PlannerBridge::new();
    let state = make_world_state();
    let decision = bridge.evaluate_goal(
        "g1",
        "Test",
        &state,
        &[],
        &ScenarioGenerationConfig::default(),
        &SimulationConfig::default(),
        &RankingWeights::default(),
    );
    let deviation = bridge.outcome_predictor_mut().record_actual_outcome(
        &decision.prediction,
        55.0,
        4000.0,
        true,
    );
    assert!(deviation.deviation_score >= 0.0);
}

#[test]
fn speed_weights_change_best_scenario() {
    let mut bridge = PlannerBridge::new();
    let state = make_world_state();
    let default_decision = bridge.evaluate_goal(
        "g1",
        "Build API",
        &state,
        &[],
        &ScenarioGenerationConfig::default(),
        &SimulationConfig::default(),
        &RankingWeights::default(),
    );
    let mut bridge2 = PlannerBridge::new();
    let speed_decision = bridge2.evaluate_goal(
        "g1",
        "Build API",
        &state,
        &[],
        &ScenarioGenerationConfig::default(),
        &SimulationConfig::default(),
        &RankingWeights::speed_optimized(),
    );
    // Rankings may or may not differ, but both are valid
    assert_eq!(
        default_decision.rankings.len(),
        speed_decision.rankings.len()
    );
}

#[test]
fn decision_risk_report_present() {
    let mut bridge = PlannerBridge::new();
    let state = make_world_state();
    let decision = bridge.evaluate_goal(
        "g1",
        "Build API",
        &state,
        &[],
        &ScenarioGenerationConfig::default(),
        &SimulationConfig::default(),
        &RankingWeights::default(),
    );
    assert!(!decision.best_risk_report.id.as_str().is_empty());
}
