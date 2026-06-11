use crate::models::benchmark::BenchmarkResult;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait BenchmarkRepository: Send + Sync {
    async fn save(&self, result: &BenchmarkResult) -> anyhow::Result<()>;
    async fn get_by_model_id(&self, model_id: Uuid) -> anyhow::Result<Vec<BenchmarkResult>>;
}
