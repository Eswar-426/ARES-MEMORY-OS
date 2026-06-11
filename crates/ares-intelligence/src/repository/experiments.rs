use async_trait::async_trait;
use uuid::Uuid;

// Placeholder
#[async_trait]
pub trait ExperimentRepository: Send + Sync {
    async fn record_experiment_run(
        &self,
        experiment_id: Uuid,
        winner_id: Uuid,
    ) -> anyhow::Result<()>;
}
