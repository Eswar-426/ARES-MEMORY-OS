use crate::runtime::queue::models::WorkflowQueueItem;
use crate::runtime::execution::models::WorkflowExecutionStep;

#[async_trait::async_trait]
pub trait ExecutionPlanner: Send + Sync {
    async fn plan(&self, item: &WorkflowQueueItem) -> Result<Vec<WorkflowExecutionStep>, ares_core::AresError>;
    async fn replan(&self, item: &WorkflowQueueItem, failed_step: &WorkflowExecutionStep) -> Result<Vec<WorkflowExecutionStep>, ares_core::AresError>;
    async fn recover(&self, item: &WorkflowQueueItem) -> Result<Vec<WorkflowExecutionStep>, ares_core::AresError>;
}

pub struct DefaultExecutionPlanner;

#[async_trait::async_trait]
impl ExecutionPlanner for DefaultExecutionPlanner {
    async fn plan(&self, _item: &WorkflowQueueItem) -> Result<Vec<WorkflowExecutionStep>, ares_core::AresError> {
        // Default planner just creates a single monolithic step
        Ok(vec![])
    }

    async fn replan(&self, _item: &WorkflowQueueItem, _failed_step: &WorkflowExecutionStep) -> Result<Vec<WorkflowExecutionStep>, ares_core::AresError> {
        // Default just retries the whole thing
        Ok(vec![])
    }

    async fn recover(&self, _item: &WorkflowQueueItem) -> Result<Vec<WorkflowExecutionStep>, ares_core::AresError> {
        Ok(vec![])
    }
}
