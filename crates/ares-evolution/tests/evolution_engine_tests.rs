use ares_core::id::NodeId;
use ares_core::types::evolution::{EvolutionEvent, EvolutionEventType};
use ares_evolution::EvolutionEngine;
use ares_store::db::test_helpers::test_store;
use ares_store::repositories::evolution::SqliteEvolutionRepository;
use std::sync::Arc;

#[tokio::test]
async fn test_record_and_get_evolution_event() {
    let (store, _dir) = test_store();
    let repo = Arc::new(SqliteEvolutionRepository::new(store.clone()));
    let engine = EvolutionEngine::new(repo);

    let project_id = ares_core::id::new_id();
    let target_node = NodeId::new();

    // Satisfy foreign keys
    let conn = store.get_conn().unwrap();
    conn.execute(
        "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
         VALUES (?1, 'test', '', '/tmp', 'rust', '', 'mature', 0, 0)",
        rusqlite::params![project_id],
    ).unwrap();

    conn.execute(
        "INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at)
         VALUES (?1, ?2, 'architecture', 'target', '{}', 0, 0)",
        rusqlite::params![target_node.as_str(), project_id],
    ).unwrap();

    let event = EvolutionEvent {
        id: NodeId::new(),
        target_node: target_node.clone(),
        event_type: EvolutionEventType::DriftDetected,
        occurred_at: 1000,
        actor: Some("scanner_bot".to_string()),
        rationale: Some("Detected drift in auth service".to_string()),
        evidence_ids: vec![NodeId::new()],
        confidence: 0.9,
    };

    engine
        .record_event(&project_id, &event)
        .await
        .expect("Failed to record event");

    let events = engine
        .get_events_for_node(&project_id, target_node.as_str())
        .await
        .expect("Failed to get events");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].id, event.id);
    assert_eq!(events[0].event_type, EvolutionEventType::DriftDetected);
    assert_eq!(
        events[0].rationale,
        Some("Detected drift in auth service".to_string())
    );
}
