use ares_intelligence::providers::mock::{MockProvider, MockProviderBehavior};
use ares_intelligence::providers::ModelProvider;
use ares_intelligence::sandbox::executor::SandboxExecutor;
use ares_intelligence::sandbox::limits::Limits;
use ares_intelligence::sandbox::quotas::Quotas;

#[tokio::test]
async fn test_provider_success() {
    let provider = MockProvider::new("test", MockProviderBehavior::Success("Hello".to_string()));
    let request = ares_intelligence::providers::types::ModelRequest {
        prompt: "prompt".to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
    };
    let response = provider.generate(request).await.unwrap();
    assert_eq!(response.content, "Hello");
}

#[tokio::test]
async fn test_provider_offline() {
    let provider = MockProvider::new("test", MockProviderBehavior::Offline);
    let healthy = provider.health_check().await;
    assert!(matches!(
        healthy,
        ares_intelligence::providers::types::ProviderHealthStatus::Offline
    ));
    let request = ares_intelligence::providers::types::ModelRequest {
        prompt: "prompt".to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
    };
    let response = provider.generate(request).await;
    assert!(response.is_err());
    assert_eq!(
        response.unwrap_err().to_string(),
        "Provider is currently unavailable"
    );
}

#[tokio::test]
async fn test_provider_rate_limit() {
    let provider = MockProvider::new("test", MockProviderBehavior::RateLimit);
    let request = ares_intelligence::providers::types::ModelRequest {
        prompt: "prompt".to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
    };
    let response = provider.generate(request).await;
    assert!(response.is_err());
    assert_eq!(
        response.unwrap_err().to_string(),
        "Rate limited by provider"
    );
}

#[tokio::test]
async fn test_provider_auth_failure() {
    let provider = MockProvider::new("test", MockProviderBehavior::AuthFailure);
    let request = ares_intelligence::providers::types::ModelRequest {
        prompt: "prompt".to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
    };
    let response = provider.generate(request).await;
    assert!(response.is_err());
    assert_eq!(response.unwrap_err().to_string(), "Authentication failed");
}

#[tokio::test]
async fn test_provider_malformed_payload() {
    let provider = MockProvider::new("test", MockProviderBehavior::MalformedPayload);
    let request = ares_intelligence::providers::types::ModelRequest {
        prompt: "prompt".to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
    };
    let response = provider.generate(request).await.unwrap_err();
    assert!(matches!(
        response,
        ares_intelligence::providers::ProviderError::InvalidResponse
    ));
}

#[tokio::test]
async fn test_provider_partial_response() {
    let provider = MockProvider::new("test", MockProviderBehavior::PartialResponse);
    let request = ares_intelligence::providers::types::ModelRequest {
        prompt: "prompt".to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
    };
    let response = provider.generate(request).await;
    assert!(response.is_err());
    assert_eq!(response.unwrap_err().to_string(), "Connection failed");
}

#[tokio::test]
async fn test_provider_transient_failure_recovery() {
    let provider = MockProvider::new(
        "test",
        MockProviderBehavior::SucceedAfterFailures(2, "Recovered".to_string()),
    );
    // First call fails
    let req = ares_intelligence::providers::types::ModelRequest {
        prompt: "prompt".to_string(),
        max_tokens: None,
        temperature: None,
        stream: false,
    };
    let err1 = provider.generate(req.clone()).await.unwrap_err();
    assert!(matches!(
        err1,
        ares_intelligence::providers::ProviderError::ConnectionFailed
    ));

    let err2 = provider.generate(req.clone()).await.unwrap_err();
    assert!(matches!(
        err2,
        ares_intelligence::providers::ProviderError::ConnectionFailed
    ));

    let res = provider.generate(req.clone()).await.unwrap();
    assert_eq!(res.content, "Recovered");
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
    assert_eq!(response.unwrap_err().to_string(), "Provider error: Timeout");
}
