use super::r#trait::EventBus;
use crate::events::envelope::EventEnvelope;
use crate::events::outbox::repository::OutboxRepository;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// The OutboxDispatcher periodically polls the OutboxRepository for pending events
/// and dispatches them to the configured EventBus.
pub struct OutboxDispatcher {
    outbox_repo: Arc<OutboxRepository>,
    event_bus: Arc<dyn EventBus>,
}

impl OutboxDispatcher {
    pub fn new(outbox_repo: Arc<OutboxRepository>, event_bus: Arc<dyn EventBus>) -> Self {
        Self {
            outbox_repo,
            event_bus,
        }
    }

    pub fn start(self) {
        tokio::spawn(async move {
            loop {
                match self.outbox_repo.fetch_pending(50) {
                    Ok(events) => {
                        for outbox_event in events {
                            // Deserialize payload into EventEnvelope
                            match serde_json::from_str::<EventEnvelope>(&outbox_event.payload) {
                                Ok(envelope) => {
                                    if let Err(e) = self.event_bus.publish(envelope).await {
                                        let _ = self.outbox_repo.increment_retry(&outbox_event.id);
                                        tracing::error!("Failed to publish event: {}", e);
                                    } else {
                                        let _ = self.outbox_repo.mark_published(&outbox_event.id);
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to deserialize EventEnvelope: {}", e);
                                    let _ = self.outbox_repo.mark_failed(&outbox_event.id);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch pending outbox events: {}", e);
                    }
                }
                sleep(Duration::from_millis(500)).await;
            }
        });
    }
}
