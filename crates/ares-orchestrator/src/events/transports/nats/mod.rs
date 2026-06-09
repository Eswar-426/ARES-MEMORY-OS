pub mod config;

use super::super::bus::r#trait::{EventBus, EventSubscription};
use crate::events::envelope::EventEnvelope;
use ares_core::AresError;
use async_trait::async_trait;

// In a real implementation this would hold the `async_nats::Client`
pub struct NatsEventBus {
    #[allow(dead_code)]
    config: config::NatsConfig,
}

impl NatsEventBus {
    pub fn new(config: config::NatsConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl EventBus for NatsEventBus {
    async fn publish(&self, _event: EventEnvelope) -> Result<(), AresError> {
        // Use async-nats to publish the event
        // client.publish(event.topic, payload).await
        Ok(())
    }

    async fn subscribe(&self, topic: String) -> Result<EventSubscription, AresError> {
        // Use async-nats to subscribe
        let (_, rx) = tokio::sync::mpsc::channel(10);
        Ok(EventSubscription {
            id: uuid::Uuid::new_v4().to_string(),
            topic,
            receiver: rx,
        })
    }

    async fn unsubscribe(&self, _subscription_id: String) -> Result<(), AresError> {
        Ok(())
    }
}
