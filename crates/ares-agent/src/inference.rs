use async_trait::async_trait;

use ares_core::inference::InferenceEngine;
use ares_core::AresError;

pub struct MockInferenceEngine;

#[async_trait]
impl InferenceEngine for MockInferenceEngine {
    async fn complete(&self, prompt: &str) -> Result<serde_json::Value, AresError> {
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
