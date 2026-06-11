use super::models::SelectionExplanation;
use async_trait::async_trait;

#[async_trait]
pub trait ExplanationRepository: Send + Sync {
    async fn save_explanation(&self, explanation: SelectionExplanation) -> anyhow::Result<()>;
    async fn get_explanation(&self, task_id: &str) -> anyhow::Result<Option<SelectionExplanation>>;
}
