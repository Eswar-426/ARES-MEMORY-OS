use ares_orchestrator::events::envelope::EventEnvelope;
use ares_orchestrator::events::store::repository::EventStoreRepository;
use ares_store::db::Store;
use tempfile::tempdir;

#[tokio::test]
async fn test_event_store_insert() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("ares.db");
    let store = Store::open(&db_path).unwrap();

    // Run migrations
    let mut conn = store.get_conn().unwrap();
    ares_store::migrations::run(&mut conn).unwrap();

    let repo = EventStoreRepository::new(store.clone());

    let event = EventEnvelope::new(
        "evt_1",
        "system.test",
        "TestEvent",
        serde_json::json!({"test": "data"}),
    );

    let result = repo.insert(&event);
    assert!(result.is_ok(), "Failed to insert event into store");
}
