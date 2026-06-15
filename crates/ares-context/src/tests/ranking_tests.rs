use crate::ranking::{DistanceScorer, RankingStrategy};
use ares_core::GraphNode;

fn mock_node() -> GraphNode {
    GraphNode {
        id: ares_core::NodeId::from("mock"),
        project_id: ares_core::ProjectId::from("p1"),
        node_type: ares_core::NodeType::File,
        label: "mock".to_string(),
        file_path: Some("mock.rs".to_string()),
        properties: serde_json::Value::Null,
        deleted_at: None,
        created_at: 0,
        updated_at: 0,
    }
}

#[test]
fn test_distance_scoring() {
    let scorer = DistanceScorer::new();
    let node = mock_node();
    
    let d0 = scorer.score(&node, 0);
    let d1 = scorer.score(&node, 1);
    let d2 = scorer.score(&node, 2);
    
    assert_eq!(d0, 1.0);
    assert_eq!(d1, 0.5);
    assert_eq!(d2, 0.25);
}
