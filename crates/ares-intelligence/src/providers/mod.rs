pub mod mock;

use async_trait::async_trait;

#[async_trait]
pub trait ModelProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    async fn health_check(&self) -> anyhow::Result<bool>;
    async fn generate(&self, prompt: &str) -> anyhow::Result<String>;
}
