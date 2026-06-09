use crate::models::simulation::PlanSimulationResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OptimizationObjective {
    LowestCost,
    FastestTime,
    HighestSuccessProbability,
    Balanced,
}

pub struct PlanScoringService;

impl PlanScoringService {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    /// Scores a simulated plan based on the given optimization objective.
    /// Returns a score between 0.0 and 1.0 (higher is better).
    pub fn score_plan(&self, sim: &PlanSimulationResult, objective: &OptimizationObjective) -> f64 {
        match objective {
            OptimizationObjective::LowestCost => {
                // Heuristic: Cost inversely affects score. Base budget is arbitrary for now.
                let cost_factor = (1.0 - (sim.expected_cost / 100.0)).max(0.1);
                cost_factor * sim.success_probability
            }
            OptimizationObjective::FastestTime => {
                let time_factor = (1.0 - (sim.expected_duration_seconds / 3600.0)).max(0.1);
                time_factor * sim.success_probability
            }
            OptimizationObjective::HighestSuccessProbability => sim.success_probability,
            OptimizationObjective::Balanced => {
                let cost_factor = (1.0 - (sim.expected_cost / 100.0)).max(0.1);
                let time_factor = (1.0 - (sim.expected_duration_seconds / 3600.0)).max(0.1);

                (cost_factor + time_factor + sim.success_probability) / 3.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_sim(cost: f64, duration: f64, prob: f64) -> PlanSimulationResult {
        PlanSimulationResult {
            plan_id: ares_core::id::PlanId::new(),
            expected_cost: cost,
            expected_duration_seconds: duration,
            success_probability: prob,
            risk_score: 0.1,
            simulated_at: Utc::now(),
        }
    }

    #[test]
    fn test_scoring_lowest_cost() {
        let service = PlanScoringService::new();
        let cheap = make_sim(10.0, 3600.0, 0.9);
        let expensive = make_sim(50.0, 3600.0, 0.9);

        let score1 = service.score_plan(&cheap, &OptimizationObjective::LowestCost);
        let score2 = service.score_plan(&expensive, &OptimizationObjective::LowestCost);
        assert!(score1 > score2);
    }

    #[test]
    fn test_scoring_fastest_time() {
        let service = PlanScoringService::new();
        let fast = make_sim(50.0, 60.0, 0.9);
        let slow = make_sim(50.0, 3600.0, 0.9);

        let score1 = service.score_plan(&fast, &OptimizationObjective::FastestTime);
        let score2 = service.score_plan(&slow, &OptimizationObjective::FastestTime);
        assert!(score1 > score2);
    }

    #[test]
    fn test_scoring_highest_probability() {
        let service = PlanScoringService::new();
        let safe = make_sim(50.0, 3600.0, 0.99);
        let risky = make_sim(50.0, 3600.0, 0.50);

        let score1 = service.score_plan(&safe, &OptimizationObjective::HighestSuccessProbability);
        let score2 = service.score_plan(&risky, &OptimizationObjective::HighestSuccessProbability);
        assert!(score1 > score2);
        assert_eq!(score1, 0.99);
    }
}
