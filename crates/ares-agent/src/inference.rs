use async_trait::async_trait;

#[async_trait]
pub trait ContextInferenceEngine: Send + Sync {
    async fn complete(&self, prompt: &str) -> anyhow::Result<serde_json::Value>;
}

pub struct MockInferenceEngine;

#[async_trait]
impl ContextInferenceEngine for MockInferenceEngine {
    async fn complete(&self, prompt: &str) -> anyhow::Result<serde_json::Value> {
        let estimated_tokens = prompt.len() / 4;
        Ok(serde_json::json!({
            "provider": "mock",
            "model": "mock-v1",
            "status": "ok",
            "answer": "Mock inference completed. The context was successfully assembled.",
            "usage": {
                "prompt_tokens": estimated_tokens,
                "completion_tokens": 18,
                "total_tokens": estimated_tokens + 18
            }
        }))
    }
}
