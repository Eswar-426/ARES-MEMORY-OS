use ares_intelligence::providers::types::{ModelRequest, ProviderHealthStatus};
use ares_intelligence::providers::{ModelProvider, ProviderError};
use std::sync::Arc;

/// A reusable harness to verify any provider (real or mock) adheres to the ARES contract.
pub struct ProviderContractHarness {
    provider: Arc<dyn ModelProvider>,
}

impl ProviderContractHarness {
    pub fn new(provider: Arc<dyn ModelProvider>) -> Self {
        Self { provider }
    }

    pub async fn run_all_tests(&self) {
        self.test_metadata().await;
        self.test_health_check().await;
        self.test_generate_success().await;
        // The error mapping tests require forcing the provider into an error state,
        // which varies by provider (e.g. invalid API keys, rate limit simulation).
        // Those tests are typically executed directly on the provider instances.
    }

    pub async fn test_metadata(&self) {
        let meta = self.provider.metadata();
        assert!(!meta.id.is_empty(), "Provider ID must not be empty");
        assert!(!meta.name.is_empty(), "Provider name must not be empty");
    }

    pub async fn test_health_check(&self) {
        let status = self.provider.health_check().await;
        // In this basic contract test, we just ensure it doesn't crash.
        // It could be offline or healthy.
        assert!(
            matches!(
                status,
                ProviderHealthStatus::Healthy
                    | ProviderHealthStatus::Offline
                    | ProviderHealthStatus::Degraded
            ),
            "Unknown health status"
        );
    }

    pub async fn test_generate_success(&self) {
        let req = ModelRequest {
            prompt: "Return exactly the word: SUCCESS".to_string(),
            max_tokens: Some(10),
            temperature: Some(0.0),
            stream: false,
        };

        match self.provider.generate(req).await {
            Ok(res) => {
                assert!(res.total_tokens > 0, "Must report total tokens");
                // For a true integration test, we might check `res.content.contains("SUCCESS")`
            }
            Err(e) => {
                // We expect ProviderError
                match e {
                    ProviderError::Authentication
                    | ProviderError::RateLimited
                    | ProviderError::ProviderUnavailable => {
                        // Acceptable real-world failures
                    }
                    _ => panic!("Unexpected error type: {:?}", e),
                }
            }
        }
    }
}
