use ares_core::AresError;
use ares_knowledge_graph::models::{DomainEvent, DomainEventType, ProjectionMetrics};
use ares_knowledge_graph::projection::{ProjectionEngine, ProjectionMode};
use ares_knowledge_graph::projector_registry::{
    DecisionProjector, GapProjector, ProjectorRegistry, RequirementProjector, ResolutionProjector,
};
use ares_knowledge_graph::store::KnowledgeGraphStore;
use ares_store::Store;
use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;

async fn setup_test_engine() -> (
    Arc<KnowledgeGraphStore>,
    ProjectionEngine,
    ProjectorRegistry,
) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_domain_projection.db");
    let store = Store::open(&db_path).unwrap();
    let kg_store = Arc::new(KnowledgeGraphStore::new(Arc::new(store)));

    let engine = ProjectionEngine::new(kg_store.clone());

    let mut registry = ProjectorRegistry::new();
    registry.register(Box::new(RequirementProjector));
    registry.register(Box::new(DecisionProjector));
    registry.register(Box::new(GapProjector));
    registry.register(Box::new(ResolutionProjector));

    (kg_store, engine, registry)
}

#[tokio::test]
async fn test_requirement_projection() {
    let (store, engine, registry) = setup_test_engine().await;
    let mut metrics = ProjectionMetrics::default();

    let event = DomainEvent {
        id: "EVT-1".to_string(),
        event_type: DomainEventType::RequirementCreated,
        entity_id: "REQ-AUTH".to_string(),
        timestamp: 1622500000000,
        payload: json!({
            "title": "Authentication Requirement"
        }),
    };

    engine
        .process_event(
            &event,
            ProjectionMode::Incremental,
            &registry.projectors,
            &mut metrics,
        )
        .unwrap();

    assert_eq!(store.count_nodes().unwrap(), 1);
    assert_eq!(store.count_edges().unwrap(), 0);
    assert_eq!(metrics.nodes_created, 1);
}

#[tokio::test]
async fn test_decision_projection() {
    let (store, engine, registry) = setup_test_engine().await;
    let mut metrics = ProjectionMetrics::default();

    let event = DomainEvent {
        id: "EVT-2".to_string(),
        event_type: DomainEventType::DecisionApproved,
        entity_id: "DEC-JWT".to_string(),
        timestamp: 1622500000000,
        payload: json!({
            "title": "Use JWT",
            "approved_by": "Alice"
        }),
    };

    engine
        .process_event(
            &event,
            ProjectionMode::Incremental,
            &registry.projectors,
            &mut metrics,
        )
        .unwrap();

    // 1 Decision Node + 1 Owner Node = 2 Nodes
    assert_eq!(store.count_nodes().unwrap(), 2);
    // 1 ApprovedBy Edge
    assert_eq!(store.count_edges().unwrap(), 1);
    assert_eq!(metrics.nodes_created, 2);
    assert_eq!(metrics.edges_created, 1);
}

#[tokio::test]
async fn test_gap_projection() {
    let (store, engine, registry) = setup_test_engine().await;
    let mut metrics = ProjectionMetrics::default();

    let event = DomainEvent {
        id: "EVT-3".to_string(),
        event_type: DomainEventType::GapDetected,
        entity_id: "GAP-1".to_string(),
        timestamp: 1622500000000,
        payload: json!({
            "title": "Missing Tests",
            "root_cause": "No CI pipeline"
        }),
    };

    engine
        .process_event(
            &event,
            ProjectionMode::Incremental,
            &registry.projectors,
            &mut metrics,
        )
        .unwrap();

    // 1 Gap Node + 1 RootCause Node = 2 Nodes
    assert_eq!(store.count_nodes().unwrap(), 2);
    // 1 CausedBy Edge
    assert_eq!(store.count_edges().unwrap(), 1);
}

#[tokio::test]
async fn test_replay_event_stream() {
    let (store, engine, registry) = setup_test_engine().await;
    let mut metrics = ProjectionMetrics::default();

    let events = vec![
        DomainEvent {
            id: "EVT-REQ".to_string(),
            event_type: DomainEventType::RequirementCreated,
            entity_id: "REQ-1".to_string(),
            timestamp: 1000,
            payload: json!({"title": "Req"}),
        },
        DomainEvent {
            id: "EVT-DEC".to_string(),
            event_type: DomainEventType::DecisionApproved,
            entity_id: "DEC-1".to_string(),
            timestamp: 1001,
            payload: json!({"title": "Dec", "approved_by": "Bob"}),
        },
    ];

    // First Pass
    for event in &events {
        engine
            .process_event(
                event,
                ProjectionMode::Incremental,
                &registry.projectors,
                &mut metrics,
            )
            .unwrap();
    }

    assert_eq!(store.count_nodes().unwrap(), 3); // 1 Req + 1 Dec + 1 Owner
    assert_eq!(store.count_edges().unwrap(), 1); // 1 ApprovedBy

    // Replay Pass
    for event in &events {
        engine
            .process_event(
                event,
                ProjectionMode::Incremental,
                &registry.projectors,
                &mut metrics,
            )
            .unwrap();
    }

    // Graph Unchanged
    assert_eq!(store.count_nodes().unwrap(), 3);
    assert_eq!(store.count_edges().unwrap(), 1);

    // Metrics should show duplicate events skipped
    assert_eq!(metrics.duplicate_events_skipped, 2);
}
