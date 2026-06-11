use ares_intelligence::knowledge_sync::mapper::KnowledgeMapper;
use ares_intelligence::knowledge_sync::service::KnowledgeSyncService;

#[test]
fn test_sync_learned_memory_basic() {
    let service = KnowledgeSyncService::default();
    let event = service.sync_learned_memory("test memory", 0.95).unwrap();
    assert_eq!(event.payload, "test memory");
    assert_eq!(event.confidence, 0.95);
    assert!(!event.entity_id.is_empty());
}

#[test]
fn test_sync_empty_memory() {
    let service = KnowledgeSyncService::default();
    let event = service.sync_learned_memory("", 0.5).unwrap();
    assert_eq!(event.payload, "");
    assert_eq!(event.confidence, 0.5);
}

#[test]
fn test_sync_zero_confidence() {
    let service = KnowledgeSyncService::default();
    let event = service.sync_learned_memory("low quality", 0.0).unwrap();
    assert_eq!(event.confidence, 0.0);
}

#[test]
fn test_sync_high_confidence() {
    let service = KnowledgeSyncService::default();
    let event = service.sync_learned_memory("absolute truth", 1.0).unwrap();
    assert_eq!(event.confidence, 1.0);
}

#[test]
fn test_sync_negative_confidence() {
    // Current mapping passes it through, but we ensure it doesn't crash
    let service = KnowledgeSyncService::default();
    let event = service.sync_learned_memory("bad assumption", -0.5).unwrap();
    assert_eq!(event.confidence, -0.5);
}

#[test]
fn test_sync_large_payload() {
    let service = KnowledgeSyncService::default();
    let large_memory = "A".repeat(10_000);
    let event = service.sync_learned_memory(&large_memory, 0.8).unwrap();
    assert_eq!(event.payload.len(), 10_000);
}

#[test]
fn test_mapper_generates_unique_entity_ids() {
    let mapper = KnowledgeMapper::new();
    let event1 = mapper.map_to_event("fact 1", 0.9);
    let event2 = mapper.map_to_event("fact 1", 0.9);

    // Even with exact same input, entity IDs should be unique (uuid v7)
    assert_ne!(event1.entity_id, event2.entity_id);
}

#[test]
fn test_mapper_preserves_payload() {
    let mapper = KnowledgeMapper::new();
    let payload = "Preserve this exact string with spaces and punctuation!!!";
    let event = mapper.map_to_event(payload, 0.5);
    assert_eq!(event.payload, payload);
}

#[test]
fn test_mapper_preserves_confidence() {
    let mapper = KnowledgeMapper::new();
    let event = mapper.map_to_event("test", 0.123456);
    assert_eq!(event.confidence, 0.123456);
}

#[test]
fn test_service_default_instantiation() {
    let service1 = KnowledgeSyncService::default();
    let service2 = KnowledgeSyncService::default();

    let e1 = service1.sync_learned_memory("test", 0.5).unwrap();
    let e2 = service2.sync_learned_memory("test", 0.5).unwrap();
    assert_ne!(e1.entity_id, e2.entity_id);
}

#[test]
fn test_service_custom_mapper() {
    let mapper = KnowledgeMapper::new();
    let service = KnowledgeSyncService::new(mapper);
    let event = service.sync_learned_memory("custom", 0.88).unwrap();
    assert_eq!(event.payload, "custom");
    assert_eq!(event.confidence, 0.88);
}

#[test]
fn test_sync_multiple_events_sequential() {
    let service = KnowledgeSyncService::default();
    let mut ids = std::collections::HashSet::new();

    for i in 0..100 {
        let event = service
            .sync_learned_memory(&format!("fact {}", i), 0.9)
            .unwrap();
        assert_eq!(event.payload, format!("fact {}", i));
        // Ensure no ID collisions across rapid sequential creation
        assert!(ids.insert(event.entity_id));
    }
    assert_eq!(ids.len(), 100);
}
