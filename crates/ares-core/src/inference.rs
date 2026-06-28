use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait InferenceEngine: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<Value, crate::AresError>;
}
