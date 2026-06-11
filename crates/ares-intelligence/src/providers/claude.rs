use super::{
    ModelProvider, ModelRequest, ModelResponse, ProviderError, ProviderHealthStatus,
    ProviderMetadata,
};
use crate::security::secret_manager::SecretManager;
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<ClaudeMessage>,
    max_tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    stream: bool,
}

#[derive(Deserialize)]
struct ClaudeContent {
    text: String,
}

#[derive(Deserialize)]
struct ClaudeUsage {
    input_tokens: usize,
    output_tokens: usize,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
    usage: Option<ClaudeUsage>,
}

pub struct ClaudeProvider {
    client: Client,
    secret_manager: Arc<dyn SecretManager>,
    model_name: String,
}

impl ClaudeProvider {
    pub fn new(secret_manager: Arc<dyn SecretManager>, model_name: &str) -> Self {
        Self {
            client: Client::new(),
            secret_manager,
            model_name: model_name.to_string(),
        }
    }

    async fn get_api_key(&self) -> Result<String, ProviderError> {
        self.secret_manager
            .get_secret("CLAUDE_API_KEY")
            .await
            .map_err(|_| ProviderError::Authentication)
    }
}

#[async_trait]
impl ModelProvider for ClaudeProvider {
    async fn generate(&self, request: ModelRequest) -> Result<ModelResponse, ProviderError> {
        let api_key = self.get_api_key().await?;

        let req_body = ClaudeRequest {
            model: self.model_name.clone(),
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: request.prompt.clone(),
            }],
            max_tokens: request.max_tokens.unwrap_or(4096),
            temperature: request.temperature,
            stream: false,
        };

        let res = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&req_body)
            .send()
            .await
            .map_err(|_| ProviderError::ConnectionFailed)?;

        match res.status() {
            StatusCode::OK => {
                let parsed: ClaudeResponse = res
                    .json()
                    .await
                    .map_err(|_| ProviderError::InvalidResponse)?;
                let content = parsed
                    .content
                    .into_iter()
                    .next()
                    .map(|c| c.text)
                    .unwrap_or_default();

                let usage = parsed.usage.unwrap_or(ClaudeUsage {
                    input_tokens: 0,
                    output_tokens: 0,
                });

                Ok(ModelResponse {
                    content,
                    prompt_tokens: usage.input_tokens,
                    completion_tokens: usage.output_tokens,
                    total_tokens: usage.input_tokens + usage.output_tokens,
                    streaming_supported: false,
                })
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(ProviderError::Authentication),
            StatusCode::TOO_MANY_REQUESTS => Err(ProviderError::RateLimited),
            StatusCode::BAD_REQUEST => Err(ProviderError::InvalidRequest),
            StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::BAD_GATEWAY
            | StatusCode::GATEWAY_TIMEOUT => Err(ProviderError::ProviderUnavailable),
            StatusCode::REQUEST_TIMEOUT => Err(ProviderError::Timeout),
            _ => Err(ProviderError::Unknown(format!("HTTP {}", res.status()))),
        }
    }

    async fn health_check(&self) -> ProviderHealthStatus {
        if self.get_api_key().await.is_err() {
            return ProviderHealthStatus::Offline;
        }

        // Anthropic doesn't have a simple health endpoint without auth/payload,
        // so we'll just check if the host is reachable or assume healthy if we have a key.
        ProviderHealthStatus::Healthy
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: format!("claude-{}", self.model_name),
            name: format!("Anthropic Claude ({})", self.model_name),
            version: "2023-06-01".to_string(),
            supports_streaming: false,
        }
    }
}
