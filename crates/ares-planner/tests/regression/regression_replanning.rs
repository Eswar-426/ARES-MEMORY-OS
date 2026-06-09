use ares_core::id::{GoalId, PlanId};
use ares_planner::replanning::engine::ReplanningEngine;

#[test]
fn test_regression_bug_512_replanning_preserves_context() {
    // Regression test: Replanning must ALWAYS issue a new Goal ID to break caching loops.
    let engine = ReplanningEngine::new();

    let old_plan_id = PlanId::new();
    let old_goal_id = GoalId::new();

    let new_goal_id = engine
        .trigger_replanning(&old_plan_id, &old_goal_id, "API limit reached")
        .unwrap();

    assert_ne!(new_goal_id, old_goal_id);
}
