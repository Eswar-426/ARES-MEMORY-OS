use crate::events::envelope::EventEnvelope;
use ares_core::AresError;
use async_trait::async_trait;

/// EventKnowledgeSink is responsible for taking processed events and writing them
/// into the Week 11 Knowledge Graph Memory subsystem.
#[async_trait]
pub trait EventKnowledgeSink: Send + Sync {
    async fn sink_event(&self, event: &EventEnvelope) -> Result<(), AresError>;
}

/// NoOp implementation for Week 10. Will be replaced in Week 11.
pub struct NoOpKnowledgeSink;

#[async_trait]
impl EventKnowledgeSink for NoOpKnowledgeSink {
    async fn sink_event(&self, _event: &EventEnvelope) -> Result<(), AresError> {
        // NoOp: do nothing for now
        Ok(())
    }
}
