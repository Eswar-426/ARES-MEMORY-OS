use super::ModelProvider;
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
    fn provider_id(&self) -> &str {
        &self.id
    }

    async fn health_check(&self) -> anyhow::Result<bool> {
        match self.behavior {
            MockProviderBehavior::Offline => Ok(false),
            _ => Ok(true),
        }
    }

    async fn generate(&self, _prompt: &str) -> anyhow::Result<String> {
        match &self.behavior {
            MockProviderBehavior::Success(response) => Ok(response.clone()),
            MockProviderBehavior::Timeout => {
                // Sleep for a long time to trigger timeout
                sleep(Duration::from_secs(60)).await;
                anyhow::bail!("Timeout")
            }
            MockProviderBehavior::RateLimit => {
                anyhow::bail!("HTTP 429 Too Many Requests")
            }
            MockProviderBehavior::AuthFailure => {
                anyhow::bail!("HTTP 401 Unauthorized")
            }
            MockProviderBehavior::MalformedPayload => Ok("{ malformed json".to_string()),
            MockProviderBehavior::Offline => {
                anyhow::bail!("Provider offline")
            }
            MockProviderBehavior::PartialResponse => {
                anyhow::bail!("Connection dropped unexpectedly")
            }
            MockProviderBehavior::SucceedAfterFailures(max_fails, response) => {
                let fails = self.failures.fetch_add(1, Ordering::SeqCst);
                if fails < *max_fails {
                    anyhow::bail!("Transient failure")
                } else {
                    Ok(response.clone())
                }
            }
        }
    }
}
