use ares_core::AresError;
use ares_knowledge_graph::models::{
    DomainEvent, DomainEventType, EdgeType, KnowledgeEdge, KnowledgeNode, NodeType,
    ProjectionMetrics,
};
use ares_knowledge_graph::projection::{
    GraphProjector, ProjectionBatch, ProjectionEngine, ProjectionMode,
};
use ares_knowledge_graph::store::KnowledgeGraphStore;
use ares_store::Store;
use serde_json::json;
use std::sync::Arc;

struct MockRequirementProjector;

impl GraphProjector for MockRequirementProjector {
    fn supports(&self, _event_type: &DomainEventType) -> bool {
        true
    }

    fn project(&self, event: &DomainEvent) -> Result<ProjectionBatch, AresError> {
        let mut batch = ProjectionBatch::new();

        let node = KnowledgeNode {
            id: format!("REQ-{}", event.id),
            node_type: NodeType::Requirement,
            name: format!("Requirement for {}", event.id),
            properties: json!({ "status": "active" }),
            created_at: 1000,
        };

        batch.nodes.push(node);

        let target_node = KnowledgeNode {
            id: "COMP-AUTH".to_string(),
            node_type: NodeType::Architecture,
            name: "Authentication Component".to_string(),
            properties: json!({}),
            created_at: 1000,
        };
        batch.nodes.push(target_node);

        let edge = KnowledgeEdge {
            id: "".to_string(),
            source_id: format!("REQ-{}", event.id),
            target_id: "COMP-AUTH".to_string(),
            edge_type: EdgeType::Implements,
            confidence: 1.0,
            created_at: 1000,
            properties: json!({}),
        };

        batch.edges.push(edge);
        Ok(batch)
    }
}

async fn setup_test_store() -> Arc<KnowledgeGraphStore> {
    use tempfile::tempdir;
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_projection.db");
    let store = Store::open(&db_path).unwrap();
    Arc::new(KnowledgeGraphStore::new(Arc::new(store)))
}

#[tokio::test]
async fn test_projection_replay_idempotency() {
    let store = setup_test_store().await;
    let engine = ProjectionEngine::new(store.clone());

    let event_id = "EVT-100";
    let event = DomainEvent {
        id: event_id.to_string(),
        event_type: DomainEventType::RequirementCreated,
        entity_id: event_id.to_string(),
        timestamp: 1000,
        payload: json!({}),
    };

    let projectors: Vec<Box<dyn GraphProjector>> = vec![Box::new(MockRequirementProjector)];
    let mut metrics = ProjectionMetrics::default();

    // 1. Project Once
    engine
        .process_event(
            &event,
            ProjectionMode::Incremental,
            &projectors,
            &mut metrics,
        )
        .expect("First projection failed");

    assert_eq!(store.count_nodes().unwrap(), 2);
    assert_eq!(store.count_edges().unwrap(), 1);
    assert!(store.is_event_projected(event_id).unwrap());

    // 2. Project Again (Incremental)
    engine
        .process_event(
            &event,
            ProjectionMode::Incremental,
            &projectors,
            &mut metrics,
        )
        .expect("Second projection failed");

    // Still 2 nodes and 1 edge (Ignored by ledger)
    assert_eq!(store.count_nodes().unwrap(), 2);
    assert_eq!(store.count_edges().unwrap(), 1);
}

#[tokio::test]
async fn test_projection_full_rebuild_deduplication() {
    let store = setup_test_store().await;
    let engine = ProjectionEngine::new(store.clone());

    let event_id = "EVT-200";

    let event = DomainEvent {
        id: event_id.to_string(),
        event_type: DomainEventType::RequirementCreated,
        entity_id: event_id.to_string(),
        timestamp: 1000,
        payload: json!({}),
    };

    let projectors: Vec<Box<dyn GraphProjector>> = vec![Box::new(MockRequirementProjector)];
    let mut metrics = ProjectionMetrics::default();

    // 1. Initial Projection
    engine
        .process_event(
            &event,
            ProjectionMode::Incremental,
            &projectors,
            &mut metrics,
        )
        .unwrap();
    assert_eq!(store.count_nodes().unwrap(), 2);

    // 2. Full Rebuild (ignores ledger, forces upsert)
    engine
        .process_event(
            &event,
            ProjectionMode::FullRebuild,
            &projectors,
            &mut metrics,
        )
        .unwrap();

    // Still exactly 2 nodes and 1 edge because UPSERT logic deduplicates by ID
    assert_eq!(store.count_nodes().unwrap(), 2);
    assert_eq!(store.count_edges().unwrap(), 1);
}
