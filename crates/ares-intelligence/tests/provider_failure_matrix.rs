use ares_intelligence::providers::mock::{MockProvider, MockProviderBehavior};
use ares_intelligence::providers::ModelProvider;
use ares_intelligence::sandbox::executor::SandboxExecutor;
use ares_intelligence::sandbox::limits::Limits;
use ares_intelligence::sandbox::quotas::Quotas;

#[tokio::test]
async fn test_provider_success() {
    let provider = MockProvider::new("test", MockProviderBehavior::Success("Hello".to_string()));
    let response = provider.generate("prompt").await.unwrap();
    assert_eq!(response, "Hello");
}

#[tokio::test]
async fn test_provider_offline() {
    let provider = MockProvider::new("test", MockProviderBehavior::Offline);
    let healthy = provider.health_check().await.unwrap();
    assert!(!healthy);
    let response = provider.generate("prompt").await;
    assert!(response.is_err());
    assert_eq!(response.unwrap_err().to_string(), "Provider offline");
}

#[tokio::test]
async fn test_provider_rate_limit() {
    let provider = MockProvider::new("test", MockProviderBehavior::RateLimit);
    let response = provider.generate("prompt").await;
    assert!(response.is_err());
    assert_eq!(
        response.unwrap_err().to_string(),
        "HTTP 429 Too Many Requests"
    );
}

#[tokio::test]
async fn test_provider_auth_failure() {
    let provider = MockProvider::new("test", MockProviderBehavior::AuthFailure);
    let response = provider.generate("prompt").await;
    assert!(response.is_err());
    assert_eq!(response.unwrap_err().to_string(), "HTTP 401 Unauthorized");
}

#[tokio::test]
async fn test_provider_malformed_payload() {
    let provider = MockProvider::new("test", MockProviderBehavior::MalformedPayload);
    let response = provider.generate("prompt").await.unwrap();
    assert!(response.contains("malformed"));
}

#[tokio::test]
async fn test_provider_partial_response() {
    let provider = MockProvider::new("test", MockProviderBehavior::PartialResponse);
    let response = provider.generate("prompt").await;
    assert!(response.is_err());
    assert_eq!(
        response.unwrap_err().to_string(),
        "Connection dropped unexpectedly"
    );
}

#[tokio::test]
async fn test_provider_transient_failure_recovery() {
    let provider = MockProvider::new(
        "test",
        MockProviderBehavior::SucceedAfterFailures(2, "Recovered".to_string()),
    );
    // First call fails
    let err1 = provider.generate("prompt").await.unwrap_err();
    assert_eq!(err1.to_string(), "Transient failure");
    // Second call fails
    let err2 = provider.generate("prompt").await.unwrap_err();
    assert_eq!(err2.to_string(), "Transient failure");
    // Third call succeeds
    let res = provider.generate("prompt").await.unwrap();
    assert_eq!(res, "Recovered");
}

#[tokio::test]
async fn test_sandbox_token_limit_exceeded() {
    let provider = MockProvider::new("test", MockProviderBehavior::Success("Hello".to_string()));
    let limits = Limits {
        max_requests_per_minute: 10,
        max_tokens_per_minute: 5,
    }; // 5 tokens ~ 20 chars
    let quotas = Quotas::default();
    let executor = SandboxExecutor::new(limits, quotas);

    // 25 chars > 20 chars
    let prompt = "This is a very long prompt.";
    let response = executor.execute(&provider, prompt).await;
    assert!(response.is_err());
    assert_eq!(response.unwrap_err().to_string(), "Token limit exceeded");
}

#[tokio::test]
async fn test_sandbox_timeout() {
    let provider = MockProvider::new("test", MockProviderBehavior::Timeout);
    let limits = Limits {
        max_requests_per_minute: 10,
        max_tokens_per_minute: 1000,
    };
    let quotas = Quotas::default();
    let executor = SandboxExecutor::new(limits, quotas);

    let response = executor.execute(&provider, "prompt").await;
    assert!(response.is_err());
    assert_eq!(response.unwrap_err().to_string(), "Execution timeout");
}
