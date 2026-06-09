use ares_core::AresError;
use ares_orchestrator::events::envelope::EventEnvelope;
use ares_orchestrator::events::projections::service::{Projection, ProjectionEngine};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

struct TestProjection {
    count: Arc<Mutex<u32>>,
}

#[async_trait]
impl Projection for TestProjection {
    async fn apply(&self, _event: &EventEnvelope) -> Result<(), AresError> {
        let mut count = self.count.lock().await;
        *count += 1;
        Ok(())
    }
}

#[tokio::test]
async fn test_projection_engine() {
    let mut engine = ProjectionEngine::new();
    let count = Arc::new(Mutex::new(0));

    engine.register(Box::new(TestProjection {
        count: count.clone(),
    }));

    let event = EventEnvelope::new("evt_1", "sys", "type", serde_json::json!({}));
    engine.process_event(&event).await.unwrap();

    let final_count = *count.lock().await;
    assert_eq!(final_count, 1);
}
