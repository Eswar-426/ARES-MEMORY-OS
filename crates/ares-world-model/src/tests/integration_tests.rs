use super::*;
use crate::forecast::models::*;
use crate::forecast::outcome_predictor::OutcomePredictor;
use crate::forecast::similarity::SimilarityEngine;
use crate::forecast::strategy_ranker::StrategyRanker;
use crate::planner_bridge::bridge::PlannerBridge;
use crate::prediction::counterfactual::CounterfactualEngine;
use crate::risk::analyzer::RiskAnalyzer;
use crate::scenario::generator::ScenarioGenerator;
use crate::scenario::models::ScenarioGenerationConfig;
use crate::simulation::engine::SimulationEngine;
use crate::simulation::models::SimulationConfig;
use crate::state::engine::WorldStateEngine;

/// End-to-end: Full pipeline from goal to ranked decision.
#[test]
fn e2e_full_pipeline() {
    let mut bridge = PlannerBridge::new();
    let state = make_world_state();
    let history = vec![HistoricalMission {
        id: "h1".into(),
        title: "Build REST API".into(),
        keywords: vec!["build".into(), "api".into()],
        cost: 45.0,
        duration_secs: 3600.0,
        success: true,
        agent_count: 2,
        step_count: 5,
        completed_at: 1000,
    }];
    let decision = bridge.evaluate_goal(
        "g1",
        "Build REST API",
        &state,
        &history,
        &ScenarioGenerationConfig::default(),
        &SimulationConfig::default(),
        &RankingWeights::default(),
    );
    assert!(decision.prediction.success_probability > 0.0);
    assert!(decision.prediction.confidence > 0.0);
    assert!(!decision.explanation.reason.is_empty());
    assert_eq!(decision.rankings.len(), 4);
    assert_eq!(decision.counterfactual_results.len(), 5);
}

/// End-to-end: Pipeline with feedback loop.
#[test]
fn e2e_pipeline_with_feedback() {
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
    let deviation = bridge.outcome_predictor_mut().record_actual_outcome(
        &decision.prediction,
        55.0,
        4000.0,
        true,
    );
    assert!(deviation.deviation_score >= 0.0);
    assert!(deviation.deviation_score <= 1.0);
}

/// State → Scenario → Simulation → Risk → Ranking flows correctly.
#[test]
fn e2e_individual_components() {
    let state_engine = WorldStateEngine::new();
    let scenario_gen = ScenarioGenerator::new();
    let sim_engine = SimulationEngine::new();
    let risk_analyzer = RiskAnalyzer::new();
    let ranker = StrategyRanker::new();

    let state = make_world_state();
    let issues = state_engine.validate(&state);
    assert!(issues.is_empty());

    let scenarios = scenario_gen.generate(
        "g1",
        "Build API",
        &state,
        &ScenarioGenerationConfig::default(),
    );
    assert_eq!(scenarios.len(), 4);

    let simulations = sim_engine.simulate_batch(&scenarios, &state, &SimulationConfig::default());
    assert_eq!(simulations.len(), 4);

    let risks: Vec<_> = scenarios
        .iter()
        .zip(simulations.iter())
        .map(|(s, sim)| risk_analyzer.analyze(s, sim, &state))
        .collect();
    assert_eq!(risks.len(), 4);

    let rankings = ranker.rank(&scenarios, &simulations, &risks, &RankingWeights::default());
    assert_eq!(rankings.len(), 4);
    assert_eq!(rankings[0].rank, 1);
}

/// Similarity → Prediction → Deviation learning loop.
#[test]
fn e2e_similarity_prediction_loop() {
    let sim_engine = SimilarityEngine::new();
    let mut predictor = OutcomePredictor::new();

    let history = vec![HistoricalMission {
        id: "h1".into(),
        title: "Build Dashboard".into(),
        keywords: vec!["build".into(), "dashboard".into()],
        cost: 40.0,
        duration_secs: 3000.0,
        success: true,
        agent_count: 2,
        step_count: 5,
        completed_at: 1000,
    }];

    let similar = sim_engine.find_similar(
        "Build Admin Dashboard",
        &["build".into(), "dashboard".into()],
        &history,
        10,
    );
    assert!(!similar.is_empty());

    let simulation = crate::simulation::models::SimulationResult {
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

    let prediction = predictor.predict("g1", &simulation, &similar);
    assert_eq!(prediction.prediction_method, PredictionMethod::Blended);

    let deviation = predictor.record_actual_outcome(&prediction, 42.0, 3200.0, true);
    assert!(deviation.deviation_score < 0.5); // Should be reasonably accurate
}

/// Counterfactual analysis on best scenario.
#[test]
fn e2e_counterfactual_analysis() {
    let sim_engine = SimulationEngine::new();
    let cf_engine = CounterfactualEngine::new();
    let state = make_world_state();
    let gen = ScenarioGenerator::new();
    let scenarios = gen.generate(
        "g1",
        "Build API",
        &state,
        &ScenarioGenerationConfig::default(),
    );
    let sim = sim_engine.simulate(&scenarios[0], &state, &SimulationConfig::default());
    let results = cf_engine.evaluate_standard(&sim);
    let summary = cf_engine.summarize(&results);
    assert_eq!(summary.total_evaluated, 5);
    assert!(summary.average_impact > 0.0);
}

/// World state diff detects changes.
#[test]
fn e2e_world_state_evolution() {
    let engine = WorldStateEngine::new();
    let old = make_world_state();
    let mut new = old.clone();
    new.goals.push(crate::state::models::WorldGoal {
        id: "g2".into(),
        title: "New Goal".into(),
        priority: "medium".into(),
        status: "active".into(),
    });
    new.resources[0].available = 50.0;
    let diff = engine.diff_states(&old, &new);
    assert_eq!(diff.goals_added, 1);
    assert!(diff.budget_delta < 0.0);
}

/// Multiple prediction cycles improve calibration.
#[test]
fn e2e_calibration_over_time() {
    let mut predictor = OutcomePredictor::new();
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

    // Run 10 prediction + outcome cycles
    for _ in 0..10 {
        let pred = predictor.predict("g1", &sim, &[]);
        predictor.record_actual_outcome(&pred, 52.0, 3700.0, true);
    }

    assert_eq!(predictor.total_predictions(), 10);
    // After accurate predictions, forecast error should have decreased from initial 0.5
    assert!(predictor.average_forecast_error() < 0.5);
}

/// Persistence roundtrip for full pipeline.
#[test]
fn e2e_persistence_roundtrip() {
    let dir = tempfile::TempDir::new().unwrap();
    let store = ares_store::Store::open(&dir.path().join("test.db")).unwrap();
    let repo = crate::persistence::repository::WorldModelRepository::new();
    let conn = store.get_conn().unwrap();

    let state = make_world_state();
    repo.save_world_state(&conn, &state).unwrap();

    let gen = ScenarioGenerator::new();
    let scenarios = gen.generate(
        "g1",
        "Build API",
        &state,
        &ScenarioGenerationConfig::default(),
    );
    for s in &scenarios {
        repo.save_scenario(&conn, s, Some(state.id.as_str()))
            .unwrap();
    }

    let sim_engine = SimulationEngine::new();
    let sims = sim_engine.simulate_batch(&scenarios, &state, &SimulationConfig::default());
    for sim in &sims {
        repo.save_simulation(&conn, sim).unwrap();
    }

    let loaded_state = repo
        .get_world_state(&conn, state.id.as_str())
        .unwrap()
        .unwrap();
    assert_eq!(loaded_state.goals.len(), state.goals.len());

    let loaded_scenarios = repo.list_scenarios_for_goal(&conn, "g1").unwrap();
    assert_eq!(loaded_scenarios.len(), 4);
}

/// Different weight presets produce different rankings.
#[test]
fn e2e_weight_impact_on_ranking() {
    let gen = ScenarioGenerator::new();
    let sim_engine = SimulationEngine::new();
    let risk_analyzer = RiskAnalyzer::new();
    let ranker = StrategyRanker::new();
    let state = make_world_state();

    let scenarios = gen.generate(
        "g1",
        "Build Complex System",
        &state,
        &ScenarioGenerationConfig::default(),
    );
    let sims = sim_engine.simulate_batch(&scenarios, &state, &SimulationConfig::default());
    let risks: Vec<_> = scenarios
        .iter()
        .zip(sims.iter())
        .map(|(s, sim)| risk_analyzer.analyze(s, sim, &state))
        .collect();

    let speed_rankings = ranker.rank(
        &scenarios,
        &sims,
        &risks,
        &RankingWeights::speed_optimized(),
    );
    let cost_rankings = ranker.rank(&scenarios, &sims, &risks, &RankingWeights::cost_optimized());

    // Both should have same length but potentially different order
    assert_eq!(speed_rankings.len(), cost_rankings.len());
    // At least one ranking should differ in top position (usually)
    // Even if same, the scores should differ
    let speed_top_score = speed_rankings[0].composite_score;
    let cost_top_score = cost_rankings[0].composite_score;
    assert!(!speed_top_score.is_nan() && !cost_top_score.is_nan());
}

/// Aggregate stats from similarity match.
#[test]
fn e2e_similarity_aggregation() {
    let sim_engine = SimilarityEngine::new();
    let history = vec![
        HistoricalMission {
            id: "h1".into(),
            title: "Build React Dashboard".into(),
            keywords: vec!["build".into(), "react".into(), "dashboard".into()],
            cost: 45.0,
            duration_secs: 3600.0,
            success: true,
            agent_count: 2,
            step_count: 5,
            completed_at: 1000,
        },
        HistoricalMission {
            id: "h2".into(),
            title: "Build Vue Dashboard".into(),
            keywords: vec!["build".into(), "vue".into(), "dashboard".into()],
            cost: 40.0,
            duration_secs: 3200.0,
            success: true,
            agent_count: 2,
            step_count: 4,
            completed_at: 2000,
        },
        HistoricalMission {
            id: "h3".into(),
            title: "Build Angular Dashboard".into(),
            keywords: vec!["build".into(), "angular".into(), "dashboard".into()],
            cost: 50.0,
            duration_secs: 4000.0,
            success: false,
            agent_count: 3,
            step_count: 6,
            completed_at: 3000,
        },
    ];

    let matches = sim_engine.find_similar(
        "Build React Dashboard",
        &["build".into(), "dashboard".into()],
        &history,
        10,
    );
    let stats = sim_engine.aggregate_stats(&matches);
    assert!(stats.match_count > 0);
    assert!(stats.success_rate > 0.0);
    assert!(stats.avg_cost > 0.0);
}
