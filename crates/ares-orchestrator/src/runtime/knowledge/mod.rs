use crate::runtime::execution::models::DistributedExecutionAttempt;
use ares_core::AresError;

#[async_trait::async_trait]
pub trait ExecutionKnowledgeSink: Send + Sync {
    async fn record_execution(&self, attempt: &DistributedExecutionAttempt) -> Result<(), AresError>;
}

pub struct NoOpExecutionKnowledgeSink;

#[async_trait::async_trait]
impl ExecutionKnowledgeSink for NoOpExecutionKnowledgeSink {
    async fn record_execution(&self, _attempt: &DistributedExecutionAttempt) -> Result<(), AresError> {
        // No-op for Week 9. To be implemented in Week 11 (Knowledge Graph Memory).
        Ok(())
    }
}
