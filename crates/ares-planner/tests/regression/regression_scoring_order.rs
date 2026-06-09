use ares_core::id::PlanId;
use ares_planner::evaluation::scoring::{OptimizationObjective, PlanScoringService};
use ares_planner::models::simulation::PlanSimulationResult;
use chrono::Utc;

#[test]
fn test_regression_bug_211_scoring_order_inversion() {
    // Regression test for an imaginary bug where lowest cost actually favored highest cost due to inverse math.
    let service = PlanScoringService::new();

    let cheap = PlanSimulationResult {
        plan_id: PlanId::new(),
        expected_cost: 5.0,
        expected_duration_seconds: 3600.0,
        success_probability: 0.9,
        risk_score: 0.1,
        simulated_at: Utc::now(),
    };

    let expensive = PlanSimulationResult {
        plan_id: PlanId::new(),
        expected_cost: 95.0,
        expected_duration_seconds: 3600.0,
        success_probability: 0.9,
        risk_score: 0.1,
        simulated_at: Utc::now(),
    };

    let score_cheap = service.score_plan(&cheap, &OptimizationObjective::LowestCost);
    let score_expensive = service.score_plan(&expensive, &OptimizationObjective::LowestCost);

    // Cheap MUST score significantly higher than expensive.
    assert!(score_cheap > score_expensive);

    // Ensure the difference is massive, not a floating point artifact.
    assert!((score_cheap - score_expensive) > 0.5);
}
