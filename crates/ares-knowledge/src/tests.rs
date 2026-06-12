use crate::memory_graph::{MemoryGraphEdge, MemoryGraphNode, MemorySubgraph};
use serde_json::json;

#[test]
fn entity_tests() {
    let entity = MemoryGraphNode {
        id: "entity_1".into(),
        label: "ARES Agent".into(),
        node_type: "entity".into(),
        confidence: 0.95,
        metadata: json!({"role": "coordinator"}),
    };

    assert_eq!(entity.id, "entity_1");
    assert_eq!(entity.node_type, "entity");
    assert!(entity.confidence > 0.9);
    assert_eq!(entity.metadata["role"], "coordinator");
}

#[test]
fn relationship_tests() {
    let edge = MemoryGraphEdge {
        source: "agent_1".into(),
        target: "task_1".into(),
        relationship: "ASSIGNED_TO".into(),
        confidence: 1.0,
    };

    assert_eq!(edge.source, "agent_1");
    assert_eq!(edge.target, "task_1");
    assert_eq!(edge.relationship, "ASSIGNED_TO");
    assert_eq!(edge.confidence, 1.0);
}

#[test]
fn graph_search_tests() {
    let node1 = MemoryGraphNode {
        id: "n_1".into(),
        label: "Start".into(),
        node_type: "concept".into(),
        confidence: 1.0,
        metadata: json!({}),
    };

    let node2 = MemoryGraphNode {
        id: "n_2".into(),
        label: "End".into(),
        node_type: "concept".into(),
        confidence: 0.8,
        metadata: json!({}),
    };

    let edge = MemoryGraphEdge {
        source: "n_1".into(),
        target: "n_2".into(),
        relationship: "LEADS_TO".into(),
        confidence: 0.8,
    };

    let subgraph = MemorySubgraph {
        nodes: vec![node1, node2],
        edges: vec![edge],
    };

    assert_eq!(subgraph.nodes.len(), 2);
    assert_eq!(subgraph.edges.len(), 1);

    let found_target = subgraph
        .edges
        .iter()
        .find(|e| e.source == "n_1")
        .map(|e| e.target.clone());
    assert_eq!(found_target, Some("n_2".into()));
}

#[test]
fn knowledge_ingestion_tests() {
    // Mimic ingestion by constructing a memory graph node from raw data
    let raw_data = "Observation: API latency increased";
    let extracted_node = MemoryGraphNode {
        id: "obs_1".into(),
        label: "API latency increased".into(),
        node_type: "observation".into(),
        confidence: 0.85,
        metadata: json!({"source_text": raw_data}),
    };

    assert_eq!(extracted_node.metadata["source_text"], raw_data);
    assert_eq!(extracted_node.node_type, "observation");
}

#[test]
fn knowledge_projection_tests() {
    let subgraph = MemorySubgraph {
        nodes: vec![MemoryGraphNode {
            id: "k_1".into(),
            label: "Projected Knowledge".into(),
            node_type: "principle".into(),
            confidence: 0.9,
            metadata: json!({}),
        }],
        edges: vec![],
    };

    let projection_labels: Vec<String> = subgraph.nodes.into_iter().map(|n| n.label).collect();
    assert_eq!(projection_labels, vec!["Projected Knowledge".to_string()]);
}
