use async_trait::async_trait;

// Placeholder for now
#[async_trait]
pub trait HealthRepository: Send + Sync {
    async fn record_health_event(&self, provider_id: &str, status: &str) -> anyhow::Result<()>;
}
