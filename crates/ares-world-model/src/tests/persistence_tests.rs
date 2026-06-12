use crate::forecast::models::*;
use crate::persistence::repository::WorldModelRepository;
use crate::risk::models::*;
use ares_store::Store;
use tempfile::TempDir;

fn setup_db() -> (Store, TempDir) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = dir.path().join("test.db");
    let store = Store::open(&db_path).expect("Failed to open test store");
    (store, dir)
}

#[test]
fn save_and_get_world_state() {
    let (store, _dir) = setup_db();
    let repo = WorldModelRepository::new();
    let state = super::make_world_state();
    let conn = store.get_conn().unwrap();
    repo.save_world_state(&conn, &state).unwrap();
    let loaded = repo.get_world_state(&conn, state.id.as_str()).unwrap();
    assert!(loaded.is_some());
    let loaded = loaded.unwrap();
    assert_eq!(loaded.goals.len(), state.goals.len());
}

#[test]
fn get_nonexistent_world_state() {
    let (store, _dir) = setup_db();
    let repo = WorldModelRepository::new();
    let conn = store.get_conn().unwrap();
    let loaded = repo.get_world_state(&conn, "nonexistent").unwrap();
    assert!(loaded.is_none());
}

#[test]
fn save_and_list_scenarios() {
    let (store, _dir) = setup_db();
    let repo = WorldModelRepository::new();
    let state = super::make_world_state();
    let conn = store.get_conn().unwrap();
    repo.save_world_state(&conn, &state).unwrap();

    let gen = crate::scenario::generator::ScenarioGenerator::new();
    let scenarios = gen.generate("g1", "Test", &state, &super::make_scenario_config());
    for s in &scenarios {
        repo.save_scenario(&conn, s, Some(state.id.as_str()))
            .unwrap();
    }

    let loaded = repo.list_scenarios_for_goal(&conn, "g1").unwrap();
    assert_eq!(loaded.len(), 4);
}

#[test]
fn save_simulation() {
    let (store, _dir) = setup_db();
    let repo = WorldModelRepository::new();
    let state = super::make_world_state();
    let conn = store.get_conn().unwrap();
    repo.save_world_state(&conn, &state).unwrap();

    let gen = crate::scenario::generator::ScenarioGenerator::new();
    let scenarios = gen.generate("g1", "Test", &state, &super::make_scenario_config());
    for s in &scenarios {
        repo.save_scenario(&conn, s, Some(state.id.as_str()))
            .unwrap();
    }

    let sim_engine = crate::simulation::engine::SimulationEngine::new();
    let sim = sim_engine.simulate(
        &scenarios[0],
        &state,
        &crate::simulation::models::SimulationConfig::default(),
    );
    repo.save_simulation(&conn, &sim).unwrap();
}

#[test]
fn save_risk_report() {
    let (store, _dir) = setup_db();
    let repo = WorldModelRepository::new();
    let state = super::make_world_state();
    let conn = store.get_conn().unwrap();
    repo.save_world_state(&conn, &state).unwrap();

    let gen = crate::scenario::generator::ScenarioGenerator::new();
    let scenarios = gen.generate("g1", "Test", &state, &super::make_scenario_config());
    for s in &scenarios {
        repo.save_scenario(&conn, s, Some(state.id.as_str()))
            .unwrap();
    }

    let sim_engine = crate::simulation::engine::SimulationEngine::new();
    let sim = sim_engine.simulate(
        &scenarios[0],
        &state,
        &crate::simulation::models::SimulationConfig::default(),
    );
    repo.save_simulation(&conn, &sim).unwrap();

    let analyzer = crate::risk::analyzer::RiskAnalyzer::new();
    let report = analyzer.analyze(&scenarios[0], &sim, &state);
    repo.save_risk_report(&conn, &report).unwrap();
}

#[test]
fn save_prediction() {
    let (store, _dir) = setup_db();
    let repo = WorldModelRepository::new();
    let conn = store.get_conn().unwrap();

    let state = super::make_world_state();
    repo.save_world_state(&conn, &state).unwrap();

    let gen = crate::scenario::generator::ScenarioGenerator::new();
    let scenarios = gen.generate("g1", "Test", &state, &super::make_scenario_config());
    for s in &scenarios {
        repo.save_scenario(&conn, s, Some(state.id.as_str()))
            .unwrap();
    }

    let sim_engine = crate::simulation::engine::SimulationEngine::new();
    let sim = sim_engine.simulate(
        &scenarios[0],
        &state,
        &crate::simulation::models::SimulationConfig::default(),
    );
    repo.save_simulation(&conn, &sim).unwrap();

    let mut predictor = crate::forecast::outcome_predictor::OutcomePredictor::new();
    let prediction = predictor.predict("g1", &sim, &[]);
    repo.save_prediction(&conn, &prediction).unwrap();
}

#[test]
fn save_and_list_forecast_history() {
    let (store, _dir) = setup_db();
    let repo = WorldModelRepository::new();
    let conn = store.get_conn().unwrap();

    let state = super::make_world_state();
    repo.save_world_state(&conn, &state).unwrap();

    let gen = crate::scenario::generator::ScenarioGenerator::new();
    let scenarios = gen.generate("g1", "Test", &state, &super::make_scenario_config());
    for s in &scenarios {
        repo.save_scenario(&conn, s, Some(state.id.as_str()))
            .unwrap();
    }

    let sim_engine = crate::simulation::engine::SimulationEngine::new();
    let sim = sim_engine.simulate(
        &scenarios[0],
        &state,
        &crate::simulation::models::SimulationConfig::default(),
    );
    repo.save_simulation(&conn, &sim).unwrap();

    let mut predictor = crate::forecast::outcome_predictor::OutcomePredictor::new();
    let prediction = predictor.predict("g1", &sim, &[]);
    repo.save_prediction(&conn, &prediction).unwrap();

    let deviation = predictor.record_actual_outcome(&prediction, 55.0, 4000.0, true);
    repo.save_forecast_deviation(&conn, &deviation).unwrap();

    let history = repo.list_forecast_history(&conn, 10).unwrap();
    assert_eq!(history.len(), 1);
    assert!(history[0].actual_success);
}

#[test]
fn save_and_list_strategy_rankings() {
    let (store, _dir) = setup_db();
    let repo = WorldModelRepository::new();
    let conn = store.get_conn().unwrap();

    let state = super::make_world_state();
    repo.save_world_state(&conn, &state).unwrap();

    let gen = crate::scenario::generator::ScenarioGenerator::new();
    let scenarios = gen.generate("g1", "Test", &state, &super::make_scenario_config());
    for s in &scenarios {
        repo.save_scenario(&conn, s, Some(state.id.as_str()))
            .unwrap();
    }

    let sim_engine = crate::simulation::engine::SimulationEngine::new();
    let sims = sim_engine.simulate_batch(
        &scenarios,
        &state,
        &crate::simulation::models::SimulationConfig::default(),
    );
    for sim in &sims {
        repo.save_simulation(&conn, sim).unwrap();
    }

    let risk_analyzer = crate::risk::analyzer::RiskAnalyzer::new();
    let risks: Vec<RiskReport> = scenarios
        .iter()
        .zip(sims.iter())
        .map(|(s, sim)| risk_analyzer.analyze(s, sim, &state))
        .collect();
    for r in &risks {
        repo.save_risk_report(&conn, r).unwrap();
    }

    let ranker = crate::forecast::strategy_ranker::StrategyRanker::new();
    let rankings = ranker.rank(&scenarios, &sims, &risks, &RankingWeights::default());
    for r in &rankings {
        repo.save_strategy_ranking(&conn, "g1", r).unwrap();
    }

    let loaded = repo.list_rankings_for_goal(&conn, "g1").unwrap();
    assert_eq!(loaded.len(), 4);
    assert_eq!(loaded[0].rank, 1);
}

#[test]
fn forecast_history_empty_initially() {
    let (store, _dir) = setup_db();
    let repo = WorldModelRepository::new();
    let conn = store.get_conn().unwrap();
    let history = repo.list_forecast_history(&conn, 10).unwrap();
    assert!(history.is_empty());
}

#[test]
fn rankings_empty_for_unknown_goal() {
    let (store, _dir) = setup_db();
    let repo = WorldModelRepository::new();
    let conn = store.get_conn().unwrap();
    let rankings = repo.list_rankings_for_goal(&conn, "nonexistent").unwrap();
    assert!(rankings.is_empty());
}

#[test]
fn scenarios_empty_for_unknown_goal() {
    let (store, _dir) = setup_db();
    let repo = WorldModelRepository::new();
    let conn = store.get_conn().unwrap();
    let scenarios = repo.list_scenarios_for_goal(&conn, "nonexistent").unwrap();
    assert!(scenarios.is_empty());
}

#[test]
fn world_state_resources_roundtrip() {
    let (store, _dir) = setup_db();
    let repo = WorldModelRepository::new();
    let state = super::make_world_state();
    let conn = store.get_conn().unwrap();
    repo.save_world_state(&conn, &state).unwrap();
    let loaded = repo
        .get_world_state(&conn, state.id.as_str())
        .unwrap()
        .unwrap();
    assert_eq!(loaded.resources.len(), 2);
    assert_eq!(loaded.resources[0].name, "budget");
}

#[test]
fn world_state_constraints_roundtrip() {
    let (store, _dir) = setup_db();
    let repo = WorldModelRepository::new();
    let state = super::make_world_state();
    let conn = store.get_conn().unwrap();
    repo.save_world_state(&conn, &state).unwrap();
    let loaded = repo
        .get_world_state(&conn, state.id.as_str())
        .unwrap()
        .unwrap();
    assert_eq!(loaded.constraints.len(), 1);
}

#[test]
fn multiple_forecast_deviations() {
    let (store, _dir) = setup_db();
    let repo = WorldModelRepository::new();
    let conn = store.get_conn().unwrap();

    let state = super::make_world_state();
    repo.save_world_state(&conn, &state).unwrap();
    let gen = crate::scenario::generator::ScenarioGenerator::new();
    let scenarios = gen.generate("g1", "Test", &state, &super::make_scenario_config());
    for s in &scenarios {
        repo.save_scenario(&conn, s, Some(state.id.as_str()))
            .unwrap();
    }
    let sim_engine = crate::simulation::engine::SimulationEngine::new();
    let sim = sim_engine.simulate(
        &scenarios[0],
        &state,
        &crate::simulation::models::SimulationConfig::default(),
    );
    repo.save_simulation(&conn, &sim).unwrap();

    let mut predictor = crate::forecast::outcome_predictor::OutcomePredictor::new();
    for i in 0..5 {
        let prediction = predictor.predict("g1", &sim, &[]);
        repo.save_prediction(&conn, &prediction).unwrap();
        let deviation = predictor.record_actual_outcome(
            &prediction,
            50.0 + i as f64 * 10.0,
            3600.0,
            i % 2 == 0,
        );
        repo.save_forecast_deviation(&conn, &deviation).unwrap();
    }

    let history = repo.list_forecast_history(&conn, 10).unwrap();
    assert_eq!(history.len(), 5);
}
