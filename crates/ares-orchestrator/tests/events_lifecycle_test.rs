use ares_orchestrator::events::bus::dispatcher::OutboxDispatcher;
use ares_orchestrator::events::bus::local::LocalEventBus;
use ares_orchestrator::events::bus::r#trait::EventBus;
use ares_orchestrator::events::envelope::EventEnvelope;
use ares_orchestrator::events::outbox::repository::OutboxRepository;
use ares_orchestrator::events::store::repository::EventStoreRepository;
use ares_orchestrator::events::store::service::EventStoreService;
use ares_store::db::Store;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_event_lifecycle() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("ares.db");
    let store = Store::open(&db_path).unwrap();

    let mut conn = store.get_conn().unwrap();
    ares_store::migrations::run(&mut conn).unwrap();

    let store_repo = EventStoreRepository::new(store.clone());
    let outbox_repo = Arc::new(OutboxRepository::new(store.clone()));
    let service = EventStoreService::new(store_repo, (*outbox_repo).clone());

    let bus = Arc::new(LocalEventBus::new(vec![]));
    let dispatcher = OutboxDispatcher::new(outbox_repo.clone(), bus.clone());
    dispatcher.start();

    // 1. Subscribe to the bus
    let mut sub = bus.subscribe("business.order.*".to_string()).await.unwrap();

    // 2. Publish via EventStoreService
    let event = EventEnvelope::new(
        "evt_lifecycle_1",
        "business.order.created",
        "OrderCreated",
        serde_json::json!({"order_id": "123"}),
    );
    service.append(&event).unwrap();

    // 3. Wait for dispatcher
    sleep(Duration::from_secs(1)).await;

    // 4. Check if event arrived
    let received = sub.receiver.try_recv();
    assert!(
        received.is_ok(),
        "Event was not routed through the lifecycle successfully"
    );
    assert_eq!(received.unwrap().id, "evt_lifecycle_1");
}
