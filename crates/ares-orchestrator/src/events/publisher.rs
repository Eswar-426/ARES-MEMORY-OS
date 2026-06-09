use ares_core::AresError;
use async_trait::async_trait;

#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish an event payload to a given topic.
    /// In Week 9, this is just a NoOp or Local Channel.
    /// In Week 10, this will be swapped with NATS/Kafka.
    async fn publish(&self, topic: &str, payload: &str) -> Result<(), AresError>;
}

pub struct LocalEventPublisher;

#[async_trait]
impl EventPublisher for LocalEventPublisher {
    async fn publish(&self, topic: &str, _payload: &str) -> Result<(), AresError> {
        // Just log it for now
        tracing::debug!("Published event to topic: {}", topic);
        Ok(())
    }
}
