use ares_orchestrator::events::bus::local::LocalEventBus;
use ares_orchestrator::events::bus::r#trait::EventBus;
use ares_orchestrator::events::envelope::EventEnvelope;

#[tokio::test]
async fn test_local_event_bus_routing() {
    let bus = LocalEventBus::new(vec![]);

    let mut sub1 = bus.subscribe("system.*".to_string()).await.unwrap();
    let mut sub2 = bus.subscribe("system.logs".to_string()).await.unwrap();

    let event = EventEnvelope::new(
        "evt_1",
        "system.logs",
        "LogEvent",
        serde_json::json!({"msg": "hello"}),
    );

    bus.publish(event).await.unwrap();

    // Both should receive it
    let received1 = sub1.receiver.recv().await;
    assert!(received1.is_some());
    assert_eq!(received1.unwrap().id, "evt_1");

    let received2 = sub2.receiver.recv().await;
    assert!(received2.is_some());
    assert_eq!(received2.unwrap().id, "evt_1");
}
