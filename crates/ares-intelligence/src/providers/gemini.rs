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
struct GeminiPart {
    text: String,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
    role: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    generation_config: GeminiGenerationConfig,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiContentOutput>,
}

#[derive(Deserialize)]
struct GeminiContentOutput {
    parts: Option<Vec<GeminiPartOutput>>,
}

#[derive(Deserialize)]
struct GeminiPartOutput {
    text: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiUsageMetadata {
    prompt_token_count: Option<usize>,
    candidates_token_count: Option<usize>,
    total_token_count: Option<usize>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    usage_metadata: Option<GeminiUsageMetadata>,
}

pub struct GeminiProvider {
    client: Client,
    secret_manager: Arc<dyn SecretManager>,
    model_name: String,
}

impl GeminiProvider {
    pub fn new(secret_manager: Arc<dyn SecretManager>, model_name: &str) -> Self {
        Self {
            client: Client::new(),
            secret_manager,
            model_name: model_name.to_string(),
        }
    }

    async fn get_api_key(&self) -> Result<String, ProviderError> {
        self.secret_manager
            .get_secret("GEMINI_API_KEY")
            .await
            .map_err(|_| ProviderError::Authentication)
    }
}

#[async_trait]
impl ModelProvider for GeminiProvider {
    async fn generate(&self, request: ModelRequest) -> Result<ModelResponse, ProviderError> {
        let api_key = self.get_api_key().await?;

        let req_body = GeminiRequest {
            contents: vec![GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: request.prompt.clone(),
                }],
            }],
            generation_config: GeminiGenerationConfig {
                max_output_tokens: request.max_tokens,
                temperature: request.temperature,
            },
        };

        // For Gemini, model name usually includes the prefix, e.g., "gemini-1.5-pro"
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model_name, api_key
        );

        let res = self
            .client
            .post(&url)
            .json(&req_body)
            .send()
            .await
            .map_err(|_| ProviderError::ConnectionFailed)?;

        match res.status() {
            StatusCode::OK => {
                let parsed: GeminiResponse = res
                    .json()
                    .await
                    .map_err(|_| ProviderError::InvalidResponse)?;

                let content = parsed
                    .candidates
                    .and_then(|mut c| c.pop())
                    .and_then(|c| c.content)
                    .and_then(|c| c.parts)
                    .and_then(|mut p| p.pop())
                    .and_then(|p| p.text)
                    .unwrap_or_default();

                let (p_tok, c_tok, t_tok) = if let Some(usage) = parsed.usage_metadata {
                    (
                        usage.prompt_token_count.unwrap_or(0),
                        usage.candidates_token_count.unwrap_or(0),
                        usage.total_token_count.unwrap_or(0),
                    )
                } else {
                    (0, 0, 0)
                };

                Ok(ModelResponse {
                    content,
                    prompt_tokens: p_tok,
                    completion_tokens: c_tok,
                    total_tokens: t_tok,
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

        let api_key = match self.get_api_key().await {
            Ok(k) => k,
            Err(_) => return ProviderHealthStatus::Offline,
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models?key={}",
            api_key
        );
        match self.client.get(&url).send().await {
            Ok(res) if res.status().is_success() => ProviderHealthStatus::Healthy,
            Ok(res) if res.status() == StatusCode::UNAUTHORIZED => ProviderHealthStatus::Offline,
            Ok(res) if res.status().is_server_error() => ProviderHealthStatus::Degraded,
            _ => ProviderHealthStatus::Offline,
        }
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: format!("gemini-{}", self.model_name),
            name: format!("Google Gemini ({})", self.model_name),
            version: "v1beta".to_string(),
            supports_streaming: false,
        }
    }
}
