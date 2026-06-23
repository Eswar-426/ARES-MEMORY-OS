use ares_core::AresError;
use ares_knowledge_graph::models::{DomainEvent, DomainEventType, ProjectionMetrics};
use ares_knowledge_graph::projection::{ProjectionEngine, ProjectionMode};
use ares_knowledge_graph::projector_registry::*;
use ares_knowledge_graph::store::KnowledgeGraphStore;
use ares_knowledge_graph::traversal::{MemoryTraversal, TraversalEngine};
use ares_store::Store;
use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;

async fn setup_test_engine() -> (
    Arc<KnowledgeGraphStore>,
    ProjectionEngine,
    ProjectorRegistry,
    TraversalEngine,
) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_lineage_expansion.db");
    let store = Store::open(&db_path).unwrap();
    let kg_store = Arc::new(KnowledgeGraphStore::new(Arc::new(store)));

    let engine = ProjectionEngine::new(kg_store.clone());
    let traversal = TraversalEngine::new(kg_store.clone());

    let mut registry = ProjectorRegistry::new();
    registry.register(Box::new(RequirementProjector));
    registry.register(Box::new(DecisionProjector));
    registry.register(Box::new(ArchitectureProjector));
    registry.register(Box::new(CodeArtifactProjector));
    registry.register(Box::new(TestProjector));
    registry.register(Box::new(RuntimeSignalProjector));
    registry.register(Box::new(OutcomeProjector));
    registry.register(Box::new(OwnerProjector));

    (kg_store, engine, registry, traversal)
}

#[tokio::test]
async fn test_forward_lineage_and_reverse_causality() {
    let (store, engine, registry, traversal) = setup_test_engine().await;
    let mut metrics = ProjectionMetrics::default();

    // The canonical lineage: Requirement -> Decision -> Architecture -> Code -> Test -> Runtime -> Outcome

    let events = vec![
        DomainEvent {
            id: "EVT-1".to_string(),
            event_type: DomainEventType::RequirementCreated,
            entity_id: "REQ-1".to_string(),
            timestamp: 1000,
            payload: json!({ "title": "Scalable Auth", "owner": "Alice" }),
        },
        DomainEvent {
            id: "EVT-2".to_string(),
            event_type: DomainEventType::DecisionApproved,
            entity_id: "DEC-1".to_string(),
            timestamp: 1001,
            payload: json!({ "title": "Use JWT", "requirement_id": "REQ-1", "approved_by": "Bob" }),
        },
        DomainEvent {
            id: "EVT-3".to_string(),
            event_type: DomainEventType::ArchitectureDesigned,
            entity_id: "ARCH-1".to_string(),
            timestamp: 1002,
            payload: json!({ "title": "Auth Service", "decision_id": "DEC-1", "owner": "Charlie" }),
        },
        DomainEvent {
            id: "EVT-4".to_string(),
            event_type: DomainEventType::CodeArtifactCommitted,
            entity_id: "CODE-1".to_string(),
            timestamp: 1003,
            payload: json!({ "file_path": "auth.rs", "architecture_id": "ARCH-1" }),
        },
        DomainEvent {
            id: "EVT-5".to_string(),
            event_type: DomainEventType::TestExecuted,
            entity_id: "TEST-1".to_string(),
            timestamp: 1004,
            payload: json!({ "test_name": "test_auth_flow", "code_id": "CODE-1" }),
        },
        DomainEvent {
            id: "EVT-6".to_string(),
            event_type: DomainEventType::RuntimeSignalDetected,
            entity_id: "SIGNAL-1".to_string(),
            timestamp: 1005,
            payload: json!({ "metric_name": "latency_spike", "test_id": "TEST-1" }),
        },
        DomainEvent {
            id: "EVT-7".to_string(),
            event_type: DomainEventType::OutcomeDegraded,
            entity_id: "OUTCOME-1".to_string(),
            timestamp: 1006,
            payload: json!({ "outcome_name": "User Dropoff", "signal_id": "SIGNAL-1" }),
        },
    ];

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

    // Nodes: 7 core nodes + 3 owners (OWNER-ALICE, OWNER-BOB, OWNER-CHARLIE) = 10 nodes
    assert_eq!(store.count_nodes().unwrap(), 10);

    // Edges:
    // REQ->DEC, DEC->ARCH, ARCH->CODE, CODE->TEST, TEST->SIGNAL, SIGNAL->OUTCOME (6 edges)
    // REQ->ALICE, DEC->BOB, ARCH->CHARLIE (3 edges)
    // Total = 9 edges
    assert_eq!(store.count_edges().unwrap(), 9);

    // Forward Lineage Test
    let downstream = traversal.downstream("REQ-1", 10).unwrap();
    let downstream_ids: std::collections::HashSet<_> =
        downstream.nodes.iter().map(|n| n.id.clone()).collect();

    assert!(downstream_ids.contains("DEC-1"));
    assert!(downstream_ids.contains("ARCH-1"));
    assert!(downstream_ids.contains("CODE-1"));
    assert!(downstream_ids.contains("TEST-1"));
    assert!(downstream_ids.contains("SIGNAL-1"));
    assert!(downstream_ids.contains("OUTCOME-1"));
    assert!(downstream_ids.contains("OWNER-ALICE")); // Owner of REQ-1
                                                     // Wait, are owners downstream of REQ? Yes, REQ -> OwnedBy -> ALICE

    // Reverse Causality Test (What caused the User Dropoff?)
    let upstream = traversal.upstream("OUTCOME-1", 10).unwrap();
    let upstream_ids: std::collections::HashSet<_> =
        upstream.nodes.iter().map(|n| n.id.clone()).collect();

    assert!(upstream_ids.contains("SIGNAL-1"));
    assert!(upstream_ids.contains("TEST-1"));
    assert!(upstream_ids.contains("CODE-1"));
    assert!(upstream_ids.contains("ARCH-1"));
    assert!(upstream_ids.contains("DEC-1"));
    assert!(upstream_ids.contains("REQ-1"));

    // Shortest path from REQ to OUTCOME
    let path_opt = traversal.shortest_path("REQ-1", "OUTCOME-1").unwrap();
    assert!(path_opt.is_some());
    let path = path_opt.unwrap();

    let path_ids: Vec<_> = path.nodes.iter().map(|n| n.id.clone()).collect();
    assert_eq!(
        path_ids,
        vec![
            "REQ-1",
            "DEC-1",
            "ARCH-1",
            "CODE-1",
            "TEST-1",
            "SIGNAL-1",
            "OUTCOME-1"
        ]
    );
}
