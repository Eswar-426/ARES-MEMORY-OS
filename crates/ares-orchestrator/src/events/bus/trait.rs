use crate::events::envelope::EventEnvelope;
use ares_core::AresError;
use async_trait::async_trait;
use tokio::sync::mpsc;

pub struct EventSubscription {
    pub id: String,
    pub topic: String,
    pub receiver: mpsc::Receiver<EventEnvelope>,
}

#[async_trait]
pub trait EventBus: Send + Sync {
    /// Internal method. Should only be called by the Outbox Dispatcher.
    async fn publish(&self, event: EventEnvelope) -> Result<(), AresError>;

    async fn subscribe(&self, topic: String) -> Result<EventSubscription, AresError>;

    async fn unsubscribe(&self, subscription_id: String) -> Result<(), AresError>;
}
