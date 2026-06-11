use super::{
    ModelProvider, ModelRequest, ModelResponse, ProviderError, ProviderHealthStatus,
    ProviderMetadata,
};
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
pub enum MockProviderBehavior {
    Success(String),
    Timeout,
    RateLimit,
    AuthFailure,
    MalformedPayload,
    Offline,
    PartialResponse,
    SucceedAfterFailures(usize, String),
}

pub struct MockProvider {
    id: String,
    behavior: MockProviderBehavior,
    failures: Arc<AtomicUsize>,
}

impl MockProvider {
    pub fn new(id: &str, behavior: MockProviderBehavior) -> Self {
        Self {
            id: id.to_string(),
            behavior,
            failures: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[async_trait]
impl ModelProvider for MockProvider {
    async fn generate(&self, _request: ModelRequest) -> Result<ModelResponse, ProviderError> {
        match &self.behavior {
            MockProviderBehavior::Success(response) => Ok(ModelResponse {
                content: response.clone(),
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
                streaming_supported: true,
            }),
            MockProviderBehavior::Timeout => {
                sleep(Duration::from_millis(10)).await;
                Err(ProviderError::Timeout)
            }
            MockProviderBehavior::RateLimit => Err(ProviderError::RateLimited),
            MockProviderBehavior::AuthFailure => Err(ProviderError::Authentication),
            MockProviderBehavior::MalformedPayload => Err(ProviderError::InvalidResponse),
            MockProviderBehavior::Offline => Err(ProviderError::ProviderUnavailable),
            MockProviderBehavior::PartialResponse => Err(ProviderError::ConnectionFailed),
            MockProviderBehavior::SucceedAfterFailures(max_fails, response) => {
                let fails = self.failures.fetch_add(1, Ordering::SeqCst);
                if fails < *max_fails {
                    Err(ProviderError::ConnectionFailed)
                } else {
                    Ok(ModelResponse {
                        content: response.clone(),
                        prompt_tokens: 10,
                        completion_tokens: 20,
                        total_tokens: 30,
                        streaming_supported: true,
                    })
                }
            }
        }
    }

    async fn health_check(&self) -> ProviderHealthStatus {
        match self.behavior {
            MockProviderBehavior::Offline => ProviderHealthStatus::Offline,
            _ => ProviderHealthStatus::Healthy,
        }
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: self.id.clone(),
            name: format!("Mock Provider {}", self.id),
            version: "1.0.0".to_string(),
            supports_streaming: true,
        }
    }
}
