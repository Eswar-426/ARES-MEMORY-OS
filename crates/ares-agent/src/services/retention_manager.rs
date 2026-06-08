use ares_core::AresError;
use ares_store::repositories::traits::WorkflowRepository;
use std::sync::Arc;

#[allow(dead_code)]
pub struct RetentionManager {
    repo: Arc<dyn WorkflowRepository + Send + Sync>,
    max_retention_days: u64,
}

impl RetentionManager {
    pub fn new(repo: Arc<dyn WorkflowRepository + Send + Sync>, max_retention_days: u64) -> Self {
        Self {
            repo,
            max_retention_days,
        }
    }

    /// Background job entry point to purge old records.
    pub async fn run_cleanup(&self) -> Result<(), AresError> {
        self.cleanup_executions().await?;
        self.cleanup_snapshots().await?;
        Ok(())
    }

    async fn cleanup_executions(&self) -> Result<(), AresError> {
        // In a true implementation:
        // DELETE FROM workflow_executions WHERE end_ts < now - retention
        Ok(())
    }

    async fn cleanup_snapshots(&self) -> Result<(), AresError> {
        // Delete old snapshots without active executions
        Ok(())
    }
}
