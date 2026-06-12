use crate::explain::explainer::PredictionExplainer;
use crate::explain::explainer::PredictionExplanation;
use crate::forecast::models::{
    HistoricalMission, OutcomePrediction, RankingWeights, StrategyRanking,
};
use crate::forecast::outcome_predictor::OutcomePredictor;
use crate::forecast::similarity::SimilarityEngine;
use crate::forecast::strategy_ranker::StrategyRanker;
use crate::prediction::counterfactual::CounterfactualEngine;
use crate::prediction::models::CounterfactualResult;
use crate::risk::analyzer::RiskAnalyzer;
use crate::risk::models::RiskReport;
use crate::scenario::generator::ScenarioGenerator;
use crate::scenario::models::{Scenario, ScenarioGenerationConfig};
use crate::simulation::engine::SimulationEngine;
use crate::simulation::models::{SimulationConfig, SimulationResult};
use crate::state::models::WorldState;
use serde::{Deserialize, Serialize};

/// The full decision output from the world model pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldModelDecision {
    pub goal_id: String,
    pub goal_title: String,
    pub best_scenario: Scenario,
    pub best_simulation: SimulationResult,
    pub best_risk_report: RiskReport,
    pub prediction: OutcomePrediction,
    pub rankings: Vec<StrategyRanking>,
    pub counterfactual_results: Vec<CounterfactualResult>,
    pub explanation: PredictionExplanation,
}

/// Orchestrates the full World Model prediction pipeline:
///
/// Goal → WorldState → Scenarios → Simulations → Risk Analysis
///     → Similarity → Predictions → Rankings → Counterfactuals → Explanation
///
/// Opt-in bridge — does not modify existing planner logic.
pub struct PlannerBridge {
    scenario_generator: ScenarioGenerator,
    simulation_engine: SimulationEngine,
    risk_analyzer: RiskAnalyzer,
    strategy_ranker: StrategyRanker,
    counterfactual_engine: CounterfactualEngine,
    similarity_engine: SimilarityEngine,
    outcome_predictor: OutcomePredictor,
    explainer: PredictionExplainer,
}

impl Default for PlannerBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl PlannerBridge {
    pub fn new() -> Self {
        Self {
            scenario_generator: ScenarioGenerator::new(),
            simulation_engine: SimulationEngine::new(),
            risk_analyzer: RiskAnalyzer::new(),
            strategy_ranker: StrategyRanker::new(),
            counterfactual_engine: CounterfactualEngine::new(),
            similarity_engine: SimilarityEngine::new(),
            outcome_predictor: OutcomePredictor::new(),
            explainer: PredictionExplainer::new(),
        }
    }

    /// Execute the full prediction pipeline for a goal.
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate_goal(
        &mut self,
        goal_id: &str,
        goal_title: &str,
        world_state: &WorldState,
        historical_missions: &[HistoricalMission],
        scenario_config: &ScenarioGenerationConfig,
        simulation_config: &SimulationConfig,
        ranking_weights: &RankingWeights,
    ) -> WorldModelDecision {
        // 1. Generate scenarios
        let scenarios =
            self.scenario_generator
                .generate(goal_id, goal_title, world_state, scenario_config);

        // 2. Simulate each scenario
        let simulations =
            self.simulation_engine
                .simulate_batch(&scenarios, world_state, simulation_config);

        // 3. Analyze risks for each scenario
        let risk_reports: Vec<RiskReport> = scenarios
            .iter()
            .zip(simulations.iter())
            .map(|(s, sim)| self.risk_analyzer.analyze(s, sim, world_state))
            .collect();

        // 4. Find similar missions
        let keywords = self.similarity_engine.extract_keywords(goal_title);
        let similar =
            self.similarity_engine
                .find_similar(goal_title, &keywords, historical_missions, 20);

        // 5. Rank strategies
        let rankings =
            self.strategy_ranker
                .rank(&scenarios, &simulations, &risk_reports, ranking_weights);

        // 6. Pick best scenario
        let best_idx = rankings
            .first()
            .and_then(|r| scenarios.iter().position(|s| s.id == r.scenario_id))
            .unwrap_or(0);

        let best_scenario = scenarios[best_idx].clone();
        let best_simulation = simulations[best_idx].clone();
        let best_risk_report = risk_reports[best_idx].clone();

        // 7. Predict outcome for the best scenario
        let prediction = self
            .outcome_predictor
            .predict(goal_id, &best_simulation, &similar);

        // 8. Run counterfactual analysis on the best simulation
        let counterfactual_results = self
            .counterfactual_engine
            .evaluate_standard(&best_simulation);

        // 9. Generate explanation
        let explanation = self.explainer.explain(
            &best_scenario,
            &best_simulation,
            &best_risk_report,
            &prediction,
            &rankings,
            &counterfactual_results,
        );

        WorldModelDecision {
            goal_id: goal_id.to_string(),
            goal_title: goal_title.to_string(),
            best_scenario,
            best_simulation,
            best_risk_report,
            prediction,
            rankings,
            counterfactual_results,
            explanation,
        }
    }

    /// Access the outcome predictor for recording actual outcomes.
    pub fn outcome_predictor_mut(&mut self) -> &mut OutcomePredictor {
        &mut self.outcome_predictor
    }

    /// Access the similarity engine.
    pub fn similarity_engine(&self) -> &SimilarityEngine {
        &self.similarity_engine
    }
}
