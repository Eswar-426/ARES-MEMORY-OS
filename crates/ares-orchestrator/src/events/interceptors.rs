use crate::events::envelope::EventEnvelope;
use ares_core::AresError;
use async_trait::async_trait;

/// Interceptor for hooking into the event lifecycle.
#[async_trait]
pub trait EventInterceptor: Send + Sync {
    async fn before_publish(&self, event: &mut EventEnvelope) -> Result<(), AresError>;
    async fn after_publish(&self, event: &EventEnvelope) -> Result<(), AresError>;
    async fn before_consume(&self, event: &mut EventEnvelope) -> Result<(), AresError>;
    async fn after_consume(&self, event: &EventEnvelope) -> Result<(), AresError>;
}
