use ares_core::AresError;
use ares_knowledge_graph::models::{DomainEvent, DomainEventType};
use ares_memory_evolution::engine::MemoryEvolutionEngine;
use ares_memory_evolution::models::ChangeType;
use ares_memory_evolution::store::MemoryEvolutionStore;
use ares_memory_evolution::supersession::{EntitySupersession, SupersessionEngine};
use ares_store::Store;
use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;
use uuid::Uuid;

async fn setup_test_environment() -> (
    Arc<MemoryEvolutionStore>,
    MemoryEvolutionEngine,
    SupersessionEngine,
) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_evolution.db");
    let store = Arc::new(Store::open(&db_path).unwrap());

    // Execute migrations explicitly for tests
    let mut conn = store.get_conn().unwrap();
    ares_store::migrations::run(&mut conn).unwrap();

    let evo_store = Arc::new(MemoryEvolutionStore::new(store.clone()));
    let engine = MemoryEvolutionEngine::new(evo_store.clone());
    let supersession = SupersessionEngine::new(store.clone());

    (evo_store, engine, supersession)
}

#[tokio::test]
async fn test_timeline_consistency() {
    let (store, engine, supersession) = setup_test_environment().await;

    // Create
    engine
        .process_event(&DomainEvent {
            id: Uuid::now_v7().to_string(),
            event_type: DomainEventType::RequirementCreated,
            entity_id: "REQ-1".to_string(),
            timestamp: 1000,
            payload: json!({"title": "Auth", "owner": "Alice"}),
        })
        .unwrap();

    // Update
    engine
        .process_event(&DomainEvent {
            id: Uuid::now_v7().to_string(),
            event_type: DomainEventType::RequirementUpdated,
            entity_id: "REQ-1".to_string(),
            timestamp: 2000,
            payload: json!({"title": "Auth V2", "owner": "Alice", "reason": "Security Feedback"}),
        })
        .unwrap();

    // Supersede
    supersession
        .record_supersession(&EntitySupersession {
            supersession_id: Uuid::now_v7().to_string(),
            superseded_entity_id: "REQ-1".to_string(),
            superseding_entity_id: "REQ-2".to_string(),
            entity_type: "Requirement".to_string(),
            superseded_at: 3000,
            reason: Some("Major Refactor".to_string()),
        })
        .unwrap();

    // Add superseded event to timeline for completeness
    engine
        .process_event(&DomainEvent {
            id: Uuid::now_v7().to_string(),
            event_type: DomainEventType::RequirementUpdated, // Representing superseded state change
            entity_id: "REQ-1".to_string(),
            timestamp: 3000,
            payload: json!({"status": "Superseded", "reason": "Major Refactor"}),
        })
        .unwrap();

    // Verify timeline
    let timeline = store.get_timeline("REQ-1").unwrap();
    assert_eq!(timeline.revisions.len(), 3);
    assert_eq!(timeline.revisions[0].changed_at, 1000);
    assert_eq!(timeline.revisions[0].change_type, ChangeType::Created);

    assert_eq!(timeline.revisions[1].changed_at, 2000);
    assert_eq!(timeline.revisions[1].change_type, ChangeType::Updated);
    assert_eq!(
        timeline.revisions[1].reason,
        Some("Security Feedback".to_string())
    );

    assert_eq!(timeline.revisions[2].changed_at, 3000);

    // Verify Supersession
    let replaced = supersession.what_replaced_this("REQ-1").unwrap();
    assert_eq!(replaced.len(), 1);
    assert_eq!(replaced[0].superseding_entity_id, "REQ-2");
}

#[tokio::test]
async fn test_historical_reconstruction() {
    let (_store, engine, _supersession) = setup_test_environment().await;

    // T1 -> Requirement A
    engine
        .process_event(&DomainEvent {
            id: Uuid::now_v7().to_string(),
            event_type: DomainEventType::RequirementCreated,
            entity_id: "REQ-A".to_string(),
            timestamp: 100,
            payload: json!({}),
        })
        .unwrap();

    // T2 -> Requirement A updated & Requirement B created
    engine
        .process_event(&DomainEvent {
            id: Uuid::now_v7().to_string(),
            event_type: DomainEventType::RequirementUpdated,
            entity_id: "REQ-A".to_string(),
            timestamp: 200,
            payload: json!({}),
        })
        .unwrap();

    engine
        .process_event(&DomainEvent {
            id: Uuid::now_v7().to_string(),
            event_type: DomainEventType::RequirementCreated,
            entity_id: "REQ-B".to_string(),
            timestamp: 250,
            payload: json!({}),
        })
        .unwrap();

    // T3 -> Decision 1 created
    engine
        .process_event(&DomainEvent {
            id: Uuid::now_v7().to_string(),
            event_type: DomainEventType::DecisionCreated,
            entity_id: "DEC-1".to_string(),
            timestamp: 300,
            payload: json!({}),
        })
        .unwrap();

    // Verify State at T1
    let state_t1 = engine.reconstruct_state(150).unwrap();
    assert_eq!(state_t1.entities.len(), 1);
    assert!(state_t1.entities.contains(&"REQ-A".to_string()));

    // Verify State at T2
    let state_t2 = engine.reconstruct_state(250).unwrap();
    assert_eq!(state_t2.entities.len(), 2);
    assert!(state_t2.entities.contains(&"REQ-A".to_string()));
    assert!(state_t2.entities.contains(&"REQ-B".to_string()));

    // Verify State at T3
    let state_t3 = engine.reconstruct_state(350).unwrap();
    assert_eq!(state_t3.entities.len(), 3);
    assert!(state_t3.entities.contains(&"DEC-1".to_string()));
}

#[tokio::test]
async fn test_replay_safety() {
    let (store, engine, _supersession) = setup_test_environment().await;

    let event = DomainEvent {
        id: "IDEMP-EVENT-1".to_string(),
        event_type: DomainEventType::RequirementCreated,
        entity_id: "REQ-1".to_string(),
        timestamp: 1000,
        payload: json!({"title": "Idempotency"}),
    };

    // First process
    engine.process_event(&event).unwrap();

    // Replay 100 times
    for _ in 0..100 {
        engine.process_event(&event).unwrap();
    }

    let timeline = store.get_timeline("REQ-1").unwrap();
    assert_eq!(timeline.revisions.len(), 1); // Exact exactly-once processing
}
