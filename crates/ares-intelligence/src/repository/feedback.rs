use crate::models::feedback::ExecutionFeedback;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait FeedbackRepository: Send + Sync {
    async fn save(&self, feedback: &ExecutionFeedback) -> anyhow::Result<()>;
    async fn get_by_execution_id(
        &self,
        execution_id: Uuid,
    ) -> anyhow::Result<Option<ExecutionFeedback>>;
}
