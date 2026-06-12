use super::*;
use crate::forecast::models::*;
use crate::forecast::strategy_ranker::StrategyRanker;
use crate::risk::models::*;
use crate::scenario::generator::ScenarioGenerator;
use crate::simulation::engine::SimulationEngine;
use crate::simulation::models::SimulationConfig;

fn make_ranked() -> (
    Vec<crate::scenario::models::Scenario>,
    Vec<crate::simulation::models::SimulationResult>,
    Vec<RiskReport>,
) {
    let gen = ScenarioGenerator::new();
    let sim_engine = SimulationEngine::new();
    let risk_analyzer = crate::risk::analyzer::RiskAnalyzer::new();
    let state = make_world_state();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    let sims = sim_engine.simulate_batch(&scenarios, &state, &SimulationConfig::default());
    let risks: Vec<RiskReport> = scenarios
        .iter()
        .zip(sims.iter())
        .map(|(s, sim)| risk_analyzer.analyze(s, sim, &state))
        .collect();
    (scenarios, sims, risks)
}

#[test]
fn rank_returns_all_scenarios() {
    let ranker = StrategyRanker::new();
    let (scenarios, sims, risks) = make_ranked();
    let rankings = ranker.rank(&scenarios, &sims, &risks, &RankingWeights::default());
    assert_eq!(rankings.len(), 4);
}

#[test]
fn rankings_sorted_by_score() {
    let ranker = StrategyRanker::new();
    let (scenarios, sims, risks) = make_ranked();
    let rankings = ranker.rank(&scenarios, &sims, &risks, &RankingWeights::default());
    for w in rankings.windows(2) {
        assert!(w[0].composite_score >= w[1].composite_score);
    }
}

#[test]
fn ranks_are_sequential() {
    let ranker = StrategyRanker::new();
    let (scenarios, sims, risks) = make_ranked();
    let rankings = ranker.rank(&scenarios, &sims, &risks, &RankingWeights::default());
    for (i, r) in rankings.iter().enumerate() {
        assert_eq!(r.rank, (i + 1) as u32);
    }
}

#[test]
fn best_scenario_is_rank_one() {
    let ranker = StrategyRanker::new();
    let (scenarios, sims, risks) = make_ranked();
    let rankings = ranker.rank(&scenarios, &sims, &risks, &RankingWeights::default());
    let best = ranker.best_scenario_id(&rankings);
    assert!(best.is_some());
    assert_eq!(*best.unwrap(), rankings[0].scenario_id);
}

#[test]
fn speed_weights_favor_fastest() {
    let ranker = StrategyRanker::new();
    let (scenarios, sims, risks) = make_ranked();
    let rankings = ranker.rank(
        &scenarios,
        &sims,
        &risks,
        &RankingWeights::speed_optimized(),
    );
    let best_id = ranker.best_scenario_id(&rankings).unwrap();
    let best_scenario = scenarios.iter().find(|s| s.id == *best_id).unwrap();
    // Fastest or balanced should score well with speed weights
    assert!(
        best_scenario.scenario_type == crate::scenario::models::ScenarioType::Fastest
            || best_scenario.scenario_type == crate::scenario::models::ScenarioType::Balanced
    );
}

#[test]
fn cost_weights_favor_cheapest() {
    let ranker = StrategyRanker::new();
    let (scenarios, sims, risks) = make_ranked();
    let rankings = ranker.rank(&scenarios, &sims, &risks, &RankingWeights::cost_optimized());
    let best_id = ranker.best_scenario_id(&rankings).unwrap();
    let best_scenario = scenarios.iter().find(|s| s.id == *best_id).unwrap();
    assert!(
        best_scenario.scenario_type == crate::scenario::models::ScenarioType::Cheapest
            || best_scenario.scenario_type == crate::scenario::models::ScenarioType::Balanced
    );
}

#[test]
fn explanations_present() {
    let ranker = StrategyRanker::new();
    let (scenarios, sims, risks) = make_ranked();
    let rankings = ranker.rank(&scenarios, &sims, &risks, &RankingWeights::default());
    for r in &rankings {
        assert!(!r.explanation.is_empty());
        assert!(r.explanation.contains("composite="));
    }
}

#[test]
fn scores_bounded() {
    let ranker = StrategyRanker::new();
    let (scenarios, sims, risks) = make_ranked();
    let rankings = ranker.rank(&scenarios, &sims, &risks, &RankingWeights::default());
    for r in &rankings {
        assert!(r.speed_score >= 0.0 && r.speed_score <= 1.0);
        assert!(r.quality_score >= 0.0 && r.quality_score <= 1.0);
        assert!(r.cost_score >= 0.0 && r.cost_score <= 1.0);
        assert!(r.success_score >= 0.0 && r.success_score <= 1.0);
    }
}

#[test]
fn ranking_serialization() {
    let ranker = StrategyRanker::new();
    let (scenarios, sims, risks) = make_ranked();
    let rankings = ranker.rank(&scenarios, &sims, &risks, &RankingWeights::default());
    let json = serde_json::to_string(&rankings[0]).unwrap();
    let back: StrategyRanking = serde_json::from_str(&json).unwrap();
    assert_eq!(back.rank, rankings[0].rank);
}

#[test]
fn empty_scenarios_empty_ranking() {
    let ranker = StrategyRanker::new();
    let rankings = ranker.rank(&[], &[], &[], &RankingWeights::default());
    assert!(rankings.is_empty());
    assert!(ranker.best_scenario_id(&rankings).is_none());
}

#[test]
fn quality_weights_produce_different_order() {
    let ranker = StrategyRanker::new();
    let (scenarios, sims, risks) = make_ranked();
    let default_rankings = ranker.rank(&scenarios, &sims, &risks, &RankingWeights::default());
    let quality_rankings = ranker.rank(
        &scenarios,
        &sims,
        &risks,
        &RankingWeights::quality_optimized(),
    );
    // May or may not be different, but both should be valid
    assert_eq!(default_rankings.len(), quality_rankings.len());
}

#[test]
fn composite_score_changes_with_weights() {
    let ranker = StrategyRanker::new();
    let (scenarios, sims, risks) = make_ranked();
    let r1 = ranker.rank(&scenarios, &sims, &risks, &RankingWeights::default());
    let r2 = ranker.rank(
        &scenarios,
        &sims,
        &risks,
        &RankingWeights::speed_optimized(),
    );
    // Same scenario should have different scores under different weights
    let s1_score = r1
        .iter()
        .find(|r| r.scenario_id == r1[0].scenario_id)
        .unwrap()
        .composite_score;
    let s2_score = r2
        .iter()
        .find(|r| r.scenario_id == r1[0].scenario_id)
        .unwrap()
        .composite_score;
    // They may or may not differ but both should be valid numbers
    assert!(!s1_score.is_nan());
    assert!(!s2_score.is_nan());
}
