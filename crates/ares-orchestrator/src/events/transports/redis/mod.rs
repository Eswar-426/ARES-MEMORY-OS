// Stub for Redis transport

use super::super::bus::r#trait::{EventBus, EventSubscription};
use crate::events::envelope::EventEnvelope;
use ares_core::AresError;
use async_trait::async_trait;

pub struct RedisEventBus;

#[async_trait]
impl EventBus for RedisEventBus {
    async fn publish(&self, _event: EventEnvelope) -> Result<(), AresError> {
        Err(AresError::validation("Redis transport is stubbed".to_string()))
    }

    async fn subscribe(&self, _topic: String) -> Result<EventSubscription, AresError> {
        Err(AresError::validation("Redis transport is stubbed".to_string()))
    }

    async fn unsubscribe(&self, _subscription_id: String) -> Result<(), AresError> {
        Ok(())
    }
}
