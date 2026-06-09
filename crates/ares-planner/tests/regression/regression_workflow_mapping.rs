use ares_planner::dag::models::{DagEdge, DagNode, PlanDag};
use ares_planner::workflow_mapper::translator::WorkflowTranslator;

#[test]
fn test_regression_bug_602_workflow_mapping_drops_dependencies() {
    // Regression test: Ensures that workflow translator doesn't drop complex dependencies.
    let translator = WorkflowTranslator::new();

    let mut nodes = Vec::new();
    for i in 0..5 {
        nodes.push(DagNode {
            id: format!("Node{}", i),
            title: format!("Task {}", i),
            estimated_duration: 10.0,
            cost: 1.0,
        });
    }

    // Node 4 depends on Node 0, Node 1, Node 2, and Node 3
    let mut edges = Vec::new();
    for i in 0..4 {
        edges.push(DagEdge {
            source: format!("Node{}", i),
            target: "Node4".to_string(),
        });
    }

    let dag = PlanDag { nodes, edges };

    let workflow = translator
        .translate(&dag, "Reg Test".to_string(), "Desc".to_string())
        .unwrap();

    // Find step 4
    let step_4 = workflow.steps.iter().find(|s| s.name == "Task 4").unwrap();

    // Ensure ALL 4 dependencies were preserved
    assert_eq!(step_4.depends_on.len(), 4);
}
