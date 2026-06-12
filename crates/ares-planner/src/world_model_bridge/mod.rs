//! World Model Bridge — opt-in integration between the planner and the World Model.
//!
//! This bridge allows the planner to evaluate goals through the World Model
//! prediction pipeline before generating mission DAGs. It does NOT modify
//! existing planner logic — it's a separate, opt-in path.
//!
//! Usage:
//! ```ignore
//! let bridge = WorldModelBridge::new();
//! let decision = bridge.evaluate_before_planning("goal_id", "goal_title", &world_state, ...);
//! // Use decision.best_scenario to inform mission DAG generation
//! ```

use ares_world_model::forecast::models::{HistoricalMission, RankingWeights};
use ares_world_model::planner_bridge::bridge::{PlannerBridge, WorldModelDecision};
use ares_world_model::scenario::models::ScenarioGenerationConfig;
use ares_world_model::simulation::models::SimulationConfig;
use ares_world_model::state::models::WorldState;

/// Thin adapter that wraps the World Model's PlannerBridge for use
/// within the ares-planner crate.
pub struct WorldModelBridge {
    inner: PlannerBridge,
}

impl Default for WorldModelBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldModelBridge {
    pub fn new() -> Self {
        Self {
            inner: PlannerBridge::new(),
        }
    }

    /// Evaluate a goal through the World Model before planning.
    ///
    /// Returns a `WorldModelDecision` containing the recommended scenario,
    /// simulation results, risk analysis, and prediction with explanation.
    pub fn evaluate_before_planning(
        &mut self,
        goal_id: &str,
        goal_title: &str,
        world_state: &WorldState,
        historical_missions: &[HistoricalMission],
    ) -> WorldModelDecision {
        self.inner.evaluate_goal(
            goal_id,
            goal_title,
            world_state,
            historical_missions,
            &ScenarioGenerationConfig::default(),
            &SimulationConfig::default(),
            &RankingWeights::default(),
        )
    }

    /// Evaluate with custom configuration.
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate_with_config(
        &mut self,
        goal_id: &str,
        goal_title: &str,
        world_state: &WorldState,
        historical_missions: &[HistoricalMission],
        scenario_config: &ScenarioGenerationConfig,
        simulation_config: &SimulationConfig,
        ranking_weights: &RankingWeights,
    ) -> WorldModelDecision {
        self.inner.evaluate_goal(
            goal_id,
            goal_title,
            world_state,
            historical_missions,
            scenario_config,
            simulation_config,
            ranking_weights,
        )
    }

    /// Record actual outcome after mission completion (for forecast learning).
    pub fn record_outcome(
        &mut self,
        prediction: &ares_world_model::forecast::models::OutcomePrediction,
        actual_cost: f64,
        actual_duration_secs: f64,
        actual_success: bool,
    ) -> ares_world_model::forecast::models::ForecastDeviation {
        self.inner.outcome_predictor_mut().record_actual_outcome(
            prediction,
            actual_cost,
            actual_duration_secs,
            actual_success,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ares_world_model::state::models::*;

    fn make_state() -> WorldState {
        WorldState {
            id: ares_core::WorldStateId::new(),
            goals: vec![WorldGoal {
                id: "g1".into(),
                title: "Test".into(),
                priority: "high".into(),
                status: "active".into(),
            }],
            resources: vec![WorldResource {
                name: "budget".into(),
                resource_type: ResourceType::Budget,
                available: 100.0,
                capacity: 100.0,
            }],
            active_agents: vec![WorldAgent {
                id: "a1".into(),
                name: "Agent".into(),
                role: "coder".into(),
                status: "ready".into(),
                success_rate: 0.85,
            }],
            constraints: vec![],
            snapshot_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn bridge_evaluate_produces_decision() {
        let mut bridge = WorldModelBridge::new();
        let state = make_state();
        let decision = bridge.evaluate_before_planning("g1", "Build API", &state, &[]);
        assert_eq!(decision.goal_id, "g1");
        assert!(!decision.rankings.is_empty());
    }

    #[test]
    fn bridge_evaluate_with_config() {
        let mut bridge = WorldModelBridge::new();
        let state = make_state();
        let decision = bridge.evaluate_with_config(
            "g1",
            "Build API",
            &state,
            &[],
            &ScenarioGenerationConfig::default(),
            &SimulationConfig::default(),
            &RankingWeights::speed_optimized(),
        );
        assert!(!decision.rankings.is_empty());
    }

    #[test]
    fn bridge_record_outcome() {
        let mut bridge = WorldModelBridge::new();
        let state = make_state();
        let decision = bridge.evaluate_before_planning("g1", "Build API", &state, &[]);
        let deviation = bridge.record_outcome(&decision.prediction, 50.0, 3600.0, true);
        assert!(deviation.deviation_score >= 0.0);
    }
}
