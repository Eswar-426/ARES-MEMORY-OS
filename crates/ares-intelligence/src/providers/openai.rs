use super::{
    ModelProvider, ModelRequest, ModelResponse, ProviderError, ProviderHealthStatus,
    ProviderMetadata,
};
use crate::security::secret_manager::SecretManager;
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    stream: bool,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: Option<OpenAIMessage>,
}

#[derive(Deserialize)]
struct OpenAIUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

pub struct OpenAIProvider {
    client: Client,
    secret_manager: Arc<dyn SecretManager>,
    model_name: String,
}

impl OpenAIProvider {
    pub fn new(secret_manager: Arc<dyn SecretManager>, model_name: &str) -> Self {
        Self {
            client: Client::new(),
            secret_manager,
            model_name: model_name.to_string(),
        }
    }

    async fn get_api_key(&self) -> Result<String, ProviderError> {
        self.secret_manager
            .get_secret("OPENAI_API_KEY")
            .await
            .map_err(|_| ProviderError::Authentication)
    }
}

#[async_trait]
impl ModelProvider for OpenAIProvider {
    async fn generate(&self, request: ModelRequest) -> Result<ModelResponse, ProviderError> {
        let api_key = self.get_api_key().await?;

        let req_body = OpenAIRequest {
            model: self.model_name.clone(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: request.prompt.clone(),
            }],
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            stream: false, // Forcing to false since streaming is not fully implemented
        };

        let res = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(api_key)
            .json(&req_body)
            .send()
            .await
            .map_err(|_| ProviderError::ConnectionFailed)?;

        match res.status() {
            StatusCode::OK => {
                let parsed: OpenAIResponse = res
                    .json()
                    .await
                    .map_err(|_| ProviderError::InvalidResponse)?;
                let content = parsed
                    .choices
                    .into_iter()
                    .next()
                    .and_then(|c| c.message)
                    .map(|m| m.content)
                    .unwrap_or_default();

                let usage = parsed.usage.unwrap_or(OpenAIUsage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                });

                Ok(ModelResponse {
                    content,
                    prompt_tokens: usage.prompt_tokens,
                    completion_tokens: usage.completion_tokens,
                    total_tokens: usage.total_tokens,
                    streaming_supported: false,
                })
            }
            StatusCode::UNAUTHORIZED => Err(ProviderError::Authentication),
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

        match self
            .client
            .get("https://api.openai.com/v1/models")
            .send()
            .await
        {
            Ok(res) if res.status().is_success() || res.status() == StatusCode::UNAUTHORIZED => {
                ProviderHealthStatus::Healthy
            }
            Ok(res) if res.status().is_server_error() => ProviderHealthStatus::Degraded,
            _ => ProviderHealthStatus::Offline,
        }
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: format!("openai-{}", self.model_name),
            name: format!("OpenAI ({})", self.model_name),
            version: "v1".to_string(),
            supports_streaming: false,
        }
    }
}
