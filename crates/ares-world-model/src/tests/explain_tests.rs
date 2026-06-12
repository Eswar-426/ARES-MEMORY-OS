use super::*;
use crate::explain::explainer::{PredictionExplainer, PredictionExplanation};
use crate::forecast::models::*;
use crate::planner_bridge::bridge::PlannerBridge;
use crate::scenario::models::ScenarioGenerationConfig;
use crate::simulation::models::SimulationConfig;

fn make_decision() -> crate::planner_bridge::bridge::WorldModelDecision {
    let mut bridge = PlannerBridge::new();
    let state = make_world_state();
    bridge.evaluate_goal(
        "g1",
        "Build REST API",
        &state,
        &[],
        &ScenarioGenerationConfig::default(),
        &SimulationConfig::default(),
        &RankingWeights::default(),
    )
}

#[test]
fn explanation_has_scenario() {
    let d = make_decision();
    assert!(!d.explanation.selected_scenario.is_empty());
}

#[test]
fn explanation_has_scenario_type() {
    let d = make_decision();
    assert!(!d.explanation.scenario_type.is_empty());
}

#[test]
fn explanation_has_reason() {
    let d = make_decision();
    assert!(!d.explanation.reason.is_empty());
}

#[test]
fn explanation_has_contributing_factors() {
    let d = make_decision();
    assert!(!d.explanation.contributing_factors.is_empty());
}

#[test]
fn explanation_confidence_reasons_match_prediction() {
    let d = make_decision();
    assert_eq!(
        d.explanation.confidence_reasons.len(),
        d.prediction.confidence_reasons.len()
    );
}

#[test]
fn summarize_produces_text() {
    let d = make_decision();
    let explainer = PredictionExplainer::new();
    let text = explainer.summarize(&d.explanation);
    assert!(!text.is_empty());
    assert!(text.contains("strategy"));
}

#[test]
fn explanation_serialization() {
    let d = make_decision();
    let json = serde_json::to_string(&d.explanation).unwrap();
    let back: PredictionExplanation = serde_json::from_str(&json).unwrap();
    assert_eq!(back.scenario_type, d.explanation.scenario_type);
}

#[test]
fn explanation_alternatives_present() {
    let d = make_decision();
    // With 4 scenarios, we should have 3 alternatives (all except the selected one)
    assert!(!d.explanation.alternative_scenarios.is_empty());
    assert!(d.explanation.alternative_scenarios.len() <= 3);
}

#[test]
fn alternative_has_reason() {
    let d = make_decision();
    for alt in &d.explanation.alternative_scenarios {
        assert!(!alt.reason_not_selected.is_empty());
    }
}

#[test]
fn explanation_counterfactual_insights() {
    let d = make_decision();
    // Some counterfactuals should be significant
    // (may or may not have insights depending on the simulation values)
    assert!(d.explanation.counterfactual_insights.len() <= d.counterfactual_results.len());
}
