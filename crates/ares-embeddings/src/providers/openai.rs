//! OpenAI embedding provider.
//!
//! Uses the OpenAI Embeddings API (`/v1/embeddings`) to generate vectors.
//! Requires the `OPENAI_API_KEY` environment variable.
//! Default model: `text-embedding-3-small` (1536 dimensions).

use ares_core::inference::InferenceEngine;
use ares_core::vector::{
    traits::EmbeddingProvider,
    types::{Embedding, EmbeddingMetadata},
};
use ares_core::AresError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

const DEFAULT_MODEL: &str = "text-embedding-3-small";
const DEFAULT_DIMENSIONS: u32 = 1536;
const API_URL: &str = "https://api.openai.com/v1/embeddings";

/// OpenAI embedding provider.
pub struct OpenAIEmbeddingProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    completion_model: String,
    dimensions: u32,
}

impl OpenAIEmbeddingProvider {
    /// Create a new OpenAI provider.
    ///
    /// Reads `OPENAI_API_KEY` from the environment. Returns an error if missing.
    pub fn new() -> Result<Self, AresError> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| AresError::validation("OPENAI_API_KEY environment variable not set"))?;
        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
            model: DEFAULT_MODEL.to_string(),
            completion_model: "gpt-4o-mini".to_string(),
            dimensions: DEFAULT_DIMENSIONS,
        })
    }

    /// Create a provider with a custom model and dimensions.
    pub fn with_model(
        api_key: impl Into<String>,
        model: impl Into<String>,
        completion_model: impl Into<String>,
        dimensions: u32,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model: model.into(),
            completion_model: completion_model.into(),
            dimensions,
        }
    }
}

// ─────────────────── API Types ───────────────────

#[derive(Serialize)]
struct EmbeddingRequest {
    input: Vec<String>,
    model: String,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: ErrorDetail,
}

#[derive(Deserialize)]
struct ErrorDetail {
    message: String,
}

// ─────────────────── Trait Implementation ───────────────────

#[async_trait]
impl EmbeddingProvider for OpenAIEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Embedding, AresError> {
        let results = self.embed_batch(&[text]).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| AresError::validation("OpenAI returned empty embedding response"))
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Embedding>, AresError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        debug!(model = %self.model, count = texts.len(), "Generating OpenAI embeddings");

        let request = EmbeddingRequest {
            input: texts.iter().map(|t| t.to_string()).collect(),
            model: self.model.clone(),
        };

        let response = self
            .client
            .post(API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| AresError::Database(format!("OpenAI API request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            // Try to parse error message
            let msg = serde_json::from_str::<ErrorResponse>(&body)
                .map(|e| e.error.message)
                .unwrap_or(body);
            warn!(status = %status, error = %msg, "OpenAI API error");
            return Err(AresError::Database(format!(
                "OpenAI API error ({status}): {msg}"
            )));
        }

        let resp: EmbeddingResponse = response.json().await.map_err(|e| {
            AresError::Serialization(format!("Failed to parse OpenAI response: {e}"))
        })?;

        Ok(resp
            .data
            .into_iter()
            .map(|d| Embedding::new(d.embedding, &self.model))
            .collect())
    }

    fn metadata(&self) -> EmbeddingMetadata {
        EmbeddingMetadata {
            provider: "openai".to_string(),
            model: self.model.clone(),
            dimensions: self.dimensions,
            embedding_version: 1,
        }
    }
}

#[async_trait]
impl InferenceEngine for OpenAIEmbeddingProvider {
    async fn complete(&self, prompt: &str) -> Result<serde_json::Value, AresError> {
        let request = serde_json::json!({
            "model": self.completion_model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.0
        });

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| AresError::Database(format!("OpenAI chat API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AresError::Database(format!(
                "OpenAI API error ({}): {}",
                status, body
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AresError::Serialization(format!("JSON error: {e}")))?;

        let answer = body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // Match the expected response format from ContextInferenceEngine
        Ok(serde_json::json!({
            "provider": "openai",
            "model": self.completion_model,
            "status": "ok",
            "answer": answer,
            "raw_response": body
        }))
    }
}
