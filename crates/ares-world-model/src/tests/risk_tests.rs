use super::*;
use crate::risk::analyzer::RiskAnalyzer;
use crate::risk::models::*;
use crate::scenario::generator::ScenarioGenerator;
use crate::simulation::engine::SimulationEngine;
use crate::simulation::models::SimulationConfig;

fn make_risk_inputs() -> (
    Vec<crate::scenario::models::Scenario>,
    Vec<crate::simulation::models::SimulationResult>,
) {
    let gen = ScenarioGenerator::new();
    let sim = SimulationEngine::new();
    let state = make_world_state();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    let results = sim.simulate_batch(&scenarios, &state, &SimulationConfig::default());
    (scenarios, results)
}

#[test]
fn analyze_produces_report() {
    let analyzer = RiskAnalyzer::new();
    let state = make_world_state();
    let (scenarios, sims) = make_risk_inputs();
    let report = analyzer.analyze(&scenarios[0], &sims[0], &state);
    assert!(!report.id.as_str().is_empty());
}

#[test]
fn overall_risk_level_valid() {
    let analyzer = RiskAnalyzer::new();
    let state = make_world_state();
    let (scenarios, sims) = make_risk_inputs();
    let report = analyzer.analyze(&scenarios[0], &sims[0], &state);
    // Must be one of the known levels
    let valid = ["negligible", "low", "moderate", "high", "critical"];
    assert!(valid.contains(&report.overall_risk.as_str()));
}

#[test]
fn probabilities_bounded() {
    let analyzer = RiskAnalyzer::new();
    let state = make_world_state();
    let (scenarios, sims) = make_risk_inputs();
    let report = analyzer.analyze(&scenarios[0], &sims[0], &state);
    assert!(report.failure_probability >= 0.0 && report.failure_probability <= 1.0);
    assert!(report.budget_overrun_probability >= 0.0 && report.budget_overrun_probability <= 1.0);
    assert!(report.resource_exhaustion_risk >= 0.0 && report.resource_exhaustion_risk <= 1.0);
    assert!(report.dependency_risk >= 0.0 && report.dependency_risk <= 1.0);
    assert!(report.execution_risk >= 0.0 && report.execution_risk <= 1.0);
}

#[test]
fn tight_budget_high_budget_risk() {
    let analyzer = RiskAnalyzer::new();
    let state = make_world_state_tight_budget();
    let (scenarios, sims) = make_risk_inputs();
    let report = analyzer.analyze(&scenarios[0], &sims[0], &state);
    assert!(report.budget_overrun_probability > 0.3);
}

#[test]
fn no_agents_high_resource_risk() {
    let analyzer = RiskAnalyzer::new();
    let state = make_world_state_no_agents();
    let (scenarios, sims) = make_risk_inputs();
    let report = analyzer.analyze(&scenarios[0], &sims[0], &state);
    assert!(report.resource_exhaustion_risk > 0.0);
}

#[test]
fn mitigations_non_empty_when_risks_present() {
    let analyzer = RiskAnalyzer::new();
    let state = make_world_state_tight_budget();
    let (scenarios, sims) = make_risk_inputs();
    let report = analyzer.analyze(&scenarios[0], &sims[0], &state);
    assert!(!report.mitigations.is_empty());
}

#[test]
fn overall_score_correct() {
    let report = RiskReport {
        id: ares_core::RiskReportId::new(),
        scenario_id: ares_core::ScenarioId::new(),
        overall_risk: RiskLevel::Moderate,
        failure_probability: 0.5,
        budget_overrun_probability: 0.3,
        resource_exhaustion_risk: 0.2,
        dependency_risk: 0.1,
        execution_risk: 0.1,
        risk_factors: vec![],
        mitigations: vec![],
        analyzed_at: chrono::Utc::now(),
    };
    let expected = 0.3 * 0.5 + 0.2 * 0.3 + 0.2 * 0.2 + 0.15 * 0.1 + 0.15 * 0.1;
    assert!((report.overall_score() - expected).abs() < 0.01);
}

#[test]
fn is_acceptable_moderate_or_below() {
    assert!(RiskLevel::Negligible <= RiskLevel::Moderate);
    assert!(RiskLevel::Low <= RiskLevel::Moderate);
    assert!(RiskLevel::Moderate <= RiskLevel::Moderate);
    assert!(RiskLevel::High > RiskLevel::Moderate);
}

#[test]
fn risk_level_from_score() {
    assert_eq!(RiskLevel::from_score(0.05), RiskLevel::Negligible);
    assert_eq!(RiskLevel::from_score(0.2), RiskLevel::Low);
    assert_eq!(RiskLevel::from_score(0.5), RiskLevel::Moderate);
    assert_eq!(RiskLevel::from_score(0.7), RiskLevel::High);
    assert_eq!(RiskLevel::from_score(0.9), RiskLevel::Critical);
}

#[test]
fn risk_level_numeric() {
    assert!(RiskLevel::Negligible.numeric() < RiskLevel::Low.numeric());
    assert!(RiskLevel::Low.numeric() < RiskLevel::Moderate.numeric());
    assert!(RiskLevel::Moderate.numeric() < RiskLevel::High.numeric());
    assert!(RiskLevel::High.numeric() < RiskLevel::Critical.numeric());
}

#[test]
fn risk_level_display() {
    assert_eq!(RiskLevel::Critical.to_string(), "critical");
    assert_eq!(RiskLevel::Low.to_string(), "low");
}

#[test]
fn risk_level_roundtrip() {
    for level in &[
        RiskLevel::Negligible,
        RiskLevel::Low,
        RiskLevel::Moderate,
        RiskLevel::High,
        RiskLevel::Critical,
    ] {
        assert_eq!(RiskLevel::from_str_val(level.as_str()), *level);
    }
}

#[test]
fn risk_category_roundtrip() {
    for cat in &[
        RiskCategory::Failure,
        RiskCategory::Budget,
        RiskCategory::Resource,
        RiskCategory::Dependency,
        RiskCategory::Execution,
    ] {
        assert_eq!(RiskCategory::from_str_val(cat.as_str()), *cat);
    }
}

#[test]
fn risk_factor_impact() {
    let factor = RiskFactor {
        category: RiskCategory::Failure,
        description: "test".into(),
        severity: 0.8,
        likelihood: 0.5,
    };
    assert!((factor.impact() - 0.4).abs() < 0.01);
}

#[test]
fn risk_factor_impact_clamped() {
    let factor = RiskFactor {
        category: RiskCategory::Budget,
        description: "test".into(),
        severity: 1.5,
        likelihood: 1.5,
    };
    assert!(factor.impact() <= 1.0);
}

#[test]
fn high_impact_count() {
    let report = RiskReport {
        id: ares_core::RiskReportId::new(),
        scenario_id: ares_core::ScenarioId::new(),
        overall_risk: RiskLevel::High,
        failure_probability: 0.5,
        budget_overrun_probability: 0.5,
        resource_exhaustion_risk: 0.5,
        dependency_risk: 0.5,
        execution_risk: 0.5,
        risk_factors: vec![
            RiskFactor {
                category: RiskCategory::Failure,
                description: "high".into(),
                severity: 0.9,
                likelihood: 0.9,
            },
            RiskFactor {
                category: RiskCategory::Budget,
                description: "low".into(),
                severity: 0.1,
                likelihood: 0.1,
            },
        ],
        mitigations: vec![],
        analyzed_at: chrono::Utc::now(),
    };
    assert_eq!(report.high_impact_count(), 1);
}

#[test]
fn risk_report_serialization() {
    let analyzer = RiskAnalyzer::new();
    let state = make_world_state();
    let (scenarios, sims) = make_risk_inputs();
    let report = analyzer.analyze(&scenarios[0], &sims[0], &state);
    let json = serde_json::to_string(&report).unwrap();
    let back: RiskReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.overall_risk, report.overall_risk);
}

#[test]
fn risk_mitigations_deduplicated() {
    let analyzer = RiskAnalyzer::new();
    let state = make_world_state_tight_budget();
    let (scenarios, sims) = make_risk_inputs();
    let report = analyzer.analyze(&scenarios[0], &sims[0], &state);
    let mut sorted = report.mitigations.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(report.mitigations.len(), sorted.len());
}

#[test]
fn risk_level_ordering() {
    assert!(RiskLevel::Negligible < RiskLevel::Low);
    assert!(RiskLevel::Low < RiskLevel::Moderate);
    assert!(RiskLevel::Moderate < RiskLevel::High);
    assert!(RiskLevel::High < RiskLevel::Critical);
}

#[test]
fn risk_level_as_str_roundtrip() {
    for level in &[
        RiskLevel::Negligible,
        RiskLevel::Low,
        RiskLevel::Moderate,
        RiskLevel::High,
        RiskLevel::Critical,
    ] {
        let s = level.as_str();
        assert!(!s.is_empty());
    }
}

#[test]
fn risk_report_overall_score_bounded() {
    let analyzer = RiskAnalyzer::new();
    let state = make_world_state();
    let (scenarios, sims) = make_risk_inputs();
    for (s, sim) in scenarios.iter().zip(sims.iter()) {
        let report = analyzer.analyze(s, sim, &state);
        let score = report.overall_score();
        assert!(score >= 0.0, "Score should be non-negative");
        assert!(score <= 1.0, "Score should be at most 1.0");
    }
}

#[test]
fn risk_factor_severity_bounded() {
    let analyzer = RiskAnalyzer::new();
    let state = make_world_state();
    let (scenarios, sims) = make_risk_inputs();
    let report = analyzer.analyze(&scenarios[0], &sims[0], &state);
    for factor in &report.risk_factors {
        assert!(factor.severity >= 0.0);
        assert!(factor.severity <= 1.0);
    }
}

#[test]
fn risk_factor_likelihood_bounded() {
    let analyzer = RiskAnalyzer::new();
    let state = make_world_state();
    let (scenarios, sims) = make_risk_inputs();
    let report = analyzer.analyze(&scenarios[0], &sims[0], &state);
    for factor in &report.risk_factors {
        assert!(factor.likelihood >= 0.0);
        assert!(factor.likelihood <= 1.0);
    }
}

#[test]
fn risk_report_with_many_agents() {
    let analyzer = RiskAnalyzer::new();
    let mut state = make_world_state();
    for i in 0..10 {
        state.active_agents.push(crate::state::models::WorldAgent {
            id: format!("agent_{}", i),
            name: format!("Agent {}", i),
            role: "worker".into(),
            status: "ready".into(),
            success_rate: 0.9,
        });
    }
    let (scenarios, sims) = make_risk_inputs();
    let report = analyzer.analyze(&scenarios[0], &sims[0], &state);
    // With many agents, resource risk should be lower
    assert!(report.resource_exhaustion_risk <= 0.5);
}

#[test]
fn risk_acceptable_for_low_risk() {
    let analyzer = RiskAnalyzer::new();
    let state = make_world_state();
    let (scenarios, sims) = make_risk_inputs();
    let report = analyzer.analyze(&scenarios[0], &sims[0], &state);
    if report.overall_risk <= RiskLevel::Moderate {
        assert!(report.is_acceptable());
    }
}

#[test]
fn risk_not_acceptable_for_high_risk() {
    let report = RiskReport {
        id: ares_core::RiskReportId::new(),
        scenario_id: ares_core::ScenarioId::new(),
        overall_risk: RiskLevel::High,
        failure_probability: 0.8,
        budget_overrun_probability: 0.7,
        resource_exhaustion_risk: 0.6,
        dependency_risk: 0.5,
        execution_risk: 0.4,
        risk_factors: vec![],
        mitigations: vec![],
        analyzed_at: chrono::Utc::now(),
    };
    assert!(!report.is_acceptable());
}

#[test]
fn risk_high_impact_count() {
    let report = RiskReport {
        id: ares_core::RiskReportId::new(),
        scenario_id: ares_core::ScenarioId::new(),
        overall_risk: RiskLevel::High,
        failure_probability: 0.5,
        budget_overrun_probability: 0.5,
        resource_exhaustion_risk: 0.5,
        dependency_risk: 0.5,
        execution_risk: 0.5,
        risk_factors: vec![
            RiskFactor {
                category: RiskCategory::Budget,
                description: "Over budget".into(),
                severity: 0.8,
                likelihood: 0.9,
            },
            RiskFactor {
                category: RiskCategory::Failure,
                description: "May fail".into(),
                severity: 0.3,
                likelihood: 0.5,
            },
        ],
        mitigations: vec![],
        analyzed_at: chrono::Utc::now(),
    };
    assert_eq!(report.high_impact_count(), 1);
}

#[test]
fn risk_report_debug_format() {
    let analyzer = RiskAnalyzer::new();
    let state = make_world_state();
    let (scenarios, sims) = make_risk_inputs();
    let report = analyzer.analyze(&scenarios[0], &sims[0], &state);
    let debug = format!("{:?}", report);
    assert!(!debug.is_empty());
}
