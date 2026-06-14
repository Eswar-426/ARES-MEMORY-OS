use ares_core::PlanStatus;
use ares_store::SqlitePlanRepository;
use ares_store::Store;
use ares_planner::planner::{PlannerEngine, MockPlannerProvider, topological_sort, MockTaskOutput};
use tempfile::TempDir;

fn create_test_store() -> (Store, TempDir) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");
    let store = Store::open(&db_path).unwrap();
    (store, dir)
}

#[test]
fn test_topological_sort_linear() {
    let tasks = vec![
        MockTaskOutput {
            title: "Task 1".to_string(),
            description: None,
            estimated_duration: None,
            complexity: None,
            dependencies: vec![],
        },
        MockTaskOutput {
            title: "Task 2".to_string(),
            description: None,
            estimated_duration: None,
            complexity: None,
            dependencies: vec!["Task 1".to_string()],
        },
    ];

    let sorted = topological_sort(&tasks);
    assert_eq!(sorted, vec![0, 1]);
}

#[test]
fn test_topological_sort_cycle() {
    let tasks = vec![
        MockTaskOutput {
            title: "Task 1".to_string(),
            description: None,
            estimated_duration: None,
            complexity: None,
            dependencies: vec!["Task 2".to_string()],
        },
        MockTaskOutput {
            title: "Task 2".to_string(),
            description: None,
            estimated_duration: None,
            complexity: None,
            dependencies: vec!["Task 1".to_string()],
        },
    ];

    let sorted = topological_sort(&tasks);
    assert_eq!(sorted.len(), 2);
}

#[tokio::test]
async fn test_planner_engine_oauth_plan() {
    let (store, _dir) = create_test_store();
    let provider = Box::new(MockPlannerProvider);
    let engine = PlannerEngine::new(store.clone(), provider);

    let goal = "Add OAuth Authentication to ARES";
    let details = engine.create_plan_from_goal(goal, "High").await.unwrap();

    // Verify goals
    assert_eq!(details.goal.title, goal);
    assert_eq!(details.goal.priority, "High");

    // Verify plan
    assert_eq!(details.plan.state, PlanStatus::Generated);

    // Verify milestones & tasks are populated
    assert!(!details.milestones.is_empty());
    assert!(!details.tasks.is_empty());

    // Verify database persistence by reading them back
    let repo = SqlitePlanRepository::new(store);
    let saved_details = repo.get_plan_details(&details.plan.id).unwrap().unwrap();

    assert_eq!(saved_details.plan.id, details.plan.id);
    assert_eq!(saved_details.goal.id, details.goal.id);
    assert_eq!(saved_details.milestones.len(), details.milestones.len());
    assert_eq!(saved_details.tasks.len(), details.tasks.len());
}
