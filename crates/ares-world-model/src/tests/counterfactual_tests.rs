use crate::prediction::counterfactual::CounterfactualEngine;
use crate::prediction::models::*;
use crate::simulation::models::SimulationResult;
use ares_core::{ScenarioId, SimulationId};
use chrono::Utc;

fn make_sim() -> SimulationResult {
    SimulationResult {
        id: SimulationId::new(),
        scenario_id: ScenarioId::new(),
        task_duration_secs: 3600.0,
        total_cost: 50.0,
        success_probability: 0.85,
        agent_utilization: 0.7,
        memory_usage_estimate: 100.0,
        risk_score: 0.2,
        simulated_at: Utc::now(),
    }
}

#[test]
fn evaluate_agent_failure() {
    let engine = CounterfactualEngine::new();
    let sim = make_sim();
    let cf = Counterfactual {
        id: "cf1".into(),
        counterfactual_type: CounterfactualType::AgentFailure,
        description: "Agent fails".into(),
        parameter: 0.5,
    };
    let result = engine.evaluate(&cf, &sim);
    assert!(result.adjusted_success_probability < sim.success_probability);
}

#[test]
fn evaluate_provider_unavailable() {
    let engine = CounterfactualEngine::new();
    let sim = make_sim();
    let cf = Counterfactual {
        id: "cf2".into(),
        counterfactual_type: CounterfactualType::ProviderUnavailable,
        description: "Provider down".into(),
        parameter: 0.7,
    };
    let result = engine.evaluate(&cf, &sim);
    assert!(result.adjusted_success_probability < sim.success_probability);
    assert!(result.adjusted_cost > sim.total_cost);
}

#[test]
fn evaluate_budget_reduction() {
    let engine = CounterfactualEngine::new();
    let sim = make_sim();
    let cf = Counterfactual {
        id: "cf3".into(),
        counterfactual_type: CounterfactualType::BudgetReduction,
        description: "Budget cut".into(),
        parameter: 0.5,
    };
    let result = engine.evaluate(&cf, &sim);
    assert!(result.adjusted_cost < sim.total_cost);
}

#[test]
fn evaluate_tool_access_lost() {
    let engine = CounterfactualEngine::new();
    let sim = make_sim();
    let cf = Counterfactual {
        id: "cf4".into(),
        counterfactual_type: CounterfactualType::ToolAccessLost,
        description: "Tools lost".into(),
        parameter: 0.6,
    };
    let result = engine.evaluate(&cf, &sim);
    assert!(result.adjusted_duration_secs > sim.task_duration_secs);
}

#[test]
fn evaluate_deadline_tightened() {
    let engine = CounterfactualEngine::new();
    let sim = make_sim();
    let cf = Counterfactual {
        id: "cf5".into(),
        counterfactual_type: CounterfactualType::DeadlineTightened,
        description: "Deadline halved".into(),
        parameter: 0.5,
    };
    let result = engine.evaluate(&cf, &sim);
    assert!(result.adjusted_duration_secs < sim.task_duration_secs);
    assert!(result.adjusted_cost > sim.total_cost);
}

#[test]
fn evaluate_standard_counterfactuals() {
    let engine = CounterfactualEngine::new();
    let sim = make_sim();
    let results = engine.evaluate_standard(&sim);
    assert_eq!(results.len(), 5);
}

#[test]
fn success_delta_positive_when_worse() {
    let engine = CounterfactualEngine::new();
    let sim = make_sim();
    let results = engine.evaluate_standard(&sim);
    for r in &results {
        assert!(r.success_delta() >= 0.0);
    }
}

#[test]
fn impact_score_bounded() {
    let engine = CounterfactualEngine::new();
    let sim = make_sim();
    let results = engine.evaluate_standard(&sim);
    for r in &results {
        assert!(r.impact_score >= 0.0 && r.impact_score <= 1.0);
    }
}

#[test]
fn is_significant_threshold() {
    let r = CounterfactualResult {
        counterfactual: Counterfactual {
            id: "x".into(),
            counterfactual_type: CounterfactualType::AgentFailure,
            description: "test".into(),
            parameter: 0.5,
        },
        original_success_probability: 0.8,
        adjusted_success_probability: 0.6,
        original_cost: 50.0,
        adjusted_cost: 50.0,
        original_duration_secs: 3600.0,
        adjusted_duration_secs: 3600.0,
        impact_score: 0.15,
        mitigation_suggestions: vec![],
    };
    assert!(r.is_significant());
}

#[test]
fn is_critical_threshold() {
    let r = CounterfactualResult {
        counterfactual: Counterfactual {
            id: "x".into(),
            counterfactual_type: CounterfactualType::AgentFailure,
            description: "test".into(),
            parameter: 0.5,
        },
        original_success_probability: 0.8,
        adjusted_success_probability: 0.3,
        original_cost: 50.0,
        adjusted_cost: 50.0,
        original_duration_secs: 3600.0,
        adjusted_duration_secs: 3600.0,
        impact_score: 0.35,
        mitigation_suggestions: vec![],
    };
    assert!(r.is_critical());
}

#[test]
fn summarize_counterfactuals() {
    let engine = CounterfactualEngine::new();
    let sim = make_sim();
    let results = engine.evaluate_standard(&sim);
    let summary = engine.summarize(&results);
    assert_eq!(summary.total_evaluated, 5);
    assert!(summary.most_impactful.is_some());
}

#[test]
fn summarize_empty() {
    let engine = CounterfactualEngine::new();
    let summary = engine.summarize(&[]);
    assert_eq!(summary.total_evaluated, 0);
    assert!(summary.most_impactful.is_none());
}

#[test]
fn mitigation_suggestions_present() {
    let engine = CounterfactualEngine::new();
    let sim = make_sim();
    let results = engine.evaluate_standard(&sim);
    for r in &results {
        assert!(!r.mitigation_suggestions.is_empty());
    }
}

#[test]
fn adjusted_success_clamped() {
    let engine = CounterfactualEngine::new();
    let sim = make_sim();
    let cf = Counterfactual {
        id: "cf".into(),
        counterfactual_type: CounterfactualType::ProviderUnavailable,
        description: "worst case".into(),
        parameter: 1.0,
    };
    let r = engine.evaluate(&cf, &sim);
    assert!(r.adjusted_success_probability >= 0.01);
    assert!(r.adjusted_success_probability <= 0.99);
}

#[test]
fn counterfactual_type_roundtrip() {
    for ct in CounterfactualType::standard_counterfactuals() {
        assert_eq!(CounterfactualType::from_str_val(ct.as_str()), ct);
    }
}

#[test]
fn zero_parameter_no_change() {
    let engine = CounterfactualEngine::new();
    let sim = make_sim();
    let cf = Counterfactual {
        id: "cf".into(),
        counterfactual_type: CounterfactualType::AgentFailure,
        description: "no change".into(),
        parameter: 0.0,
    };
    let r = engine.evaluate(&cf, &sim);
    assert!((r.adjusted_success_probability - sim.success_probability).abs() < 0.01);
}

#[test]
fn resource_reduction_counterfactual() {
    let engine = CounterfactualEngine::new();
    let sim = make_sim();
    let cf = Counterfactual {
        id: "cf".into(),
        counterfactual_type: CounterfactualType::ResourceReduction,
        description: "resources cut".into(),
        parameter: 0.5,
    };
    let r = engine.evaluate(&cf, &sim);
    assert!(r.adjusted_success_probability < sim.success_probability);
}

#[test]
fn quality_increase_counterfactual() {
    let engine = CounterfactualEngine::new();
    let sim = make_sim();
    let cf = Counterfactual {
        id: "cf".into(),
        counterfactual_type: CounterfactualType::QualityIncrease,
        description: "higher quality needed".into(),
        parameter: 0.5,
    };
    let r = engine.evaluate(&cf, &sim);
    assert!(r.adjusted_cost > sim.total_cost);
}

#[test]
fn counterfactual_result_serialization() {
    let engine = CounterfactualEngine::new();
    let sim = make_sim();
    let results = engine.evaluate_standard(&sim);
    let json = serde_json::to_string(&results[0]).unwrap();
    let back: CounterfactualResult = serde_json::from_str(&json).unwrap();
    assert!((back.impact_score - results[0].impact_score).abs() < 0.01);
}
