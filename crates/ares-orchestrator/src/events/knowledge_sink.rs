use crate::events::envelope::EventEnvelope;
use ares_core::AresError;
use async_trait::async_trait;

/// EventKnowledgeSink is responsible for taking processed events and writing them
/// into the Week 11 Knowledge Graph Memory subsystem.
#[async_trait]
pub trait EventKnowledgeSink: Send + Sync {
    async fn sink_event(&self, event: &EventEnvelope) -> Result<(), AresError>;
}

/// Real implementation for Week 11.
pub struct RealKnowledgeSink {
    inner: ares_knowledge::ingestion::event_sink::EventKnowledgeSinkImpl,
}

impl RealKnowledgeSink {
    pub fn new() -> Self {
        Self {
            inner: ares_knowledge::ingestion::event_sink::EventKnowledgeSinkImpl::new(),
        }
    }
}

impl Default for RealKnowledgeSink {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventKnowledgeSink for RealKnowledgeSink {
    async fn sink_event(&self, event: &EventEnvelope) -> Result<(), AresError> {
        let payload = serde_json::to_value(&event.payload).unwrap_or(serde_json::Value::Null);
        self.inner.consume_event(&event.event_type, payload).await
    }
}

// Keep NoOp for tests if needed, or alias it.
pub struct NoOpKnowledgeSink;

#[async_trait]
impl EventKnowledgeSink for NoOpKnowledgeSink {
    async fn sink_event(&self, _event: &EventEnvelope) -> Result<(), AresError> {
        Ok(())
    }
}
