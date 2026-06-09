use ares_planner::dag::models::{DagEdge, DagNode, PlanDag};
use ares_planner::dag::validator::DagValidator;

#[test]
fn test_regression_bug_405_deep_cycle_detection() {
    // Regression test for an imaginary bug where cycles deeper than 5 nodes failed to detect.
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    // Create A -> B -> C -> D -> E -> F -> G -> A (a deep cycle)
    for i in 0..7 {
        nodes.push(DagNode {
            id: format!("Node{}", i),
            title: format!("Task {}", i),
            estimated_duration: 1.0,
            cost: 1.0,
        });

        edges.push(DagEdge {
            source: format!("Node{}", i),
            target: format!("Node{}", (i + 1) % 7), // Wraps back to 0
        });
    }

    let dag = PlanDag { nodes, edges };

    let result = DagValidator::validate(&dag);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cycle detected"));
}
