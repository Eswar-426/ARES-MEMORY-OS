//! Ollama embedding provider.
//!
//! Uses the Ollama REST API (`/api/embeddings`) to generate vectors locally.
//! Default model: `nomic-embed-text` (768 dimensions).
//! Default endpoint: `http://localhost:11434`.

use ares_core::vector::{
    traits::EmbeddingProvider,
    types::{Embedding, EmbeddingMetadata},
};
use ares_core::AresError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

const DEFAULT_MODEL: &str = "nomic-embed-text";
const DEFAULT_DIMENSIONS: u32 = 768;
const DEFAULT_ENDPOINT: &str = "http://localhost:11434";

/// Ollama embedding provider for local model inference.
pub struct OllamaEmbeddingProvider {
    client: reqwest::Client,
    endpoint: String,
    model: String,
    dimensions: u32,
}

impl OllamaEmbeddingProvider {
    /// Create a new Ollama provider with defaults.
    ///
    /// Reads `OLLAMA_ENDPOINT` from the environment, falling back to localhost:11434.
    pub fn new() -> Self {
        let endpoint =
            std::env::var("OLLAMA_ENDPOINT").unwrap_or_else(|_| DEFAULT_ENDPOINT.to_string());
        Self {
            client: reqwest::Client::new(),
            endpoint,
            model: DEFAULT_MODEL.to_string(),
            dimensions: DEFAULT_DIMENSIONS,
        }
    }

    /// Create a provider with custom settings.
    pub fn with_config(
        endpoint: impl Into<String>,
        model: impl Into<String>,
        dimensions: u32,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint: endpoint.into(),
            model: model.into(),
            dimensions,
        }
    }
}

impl Default for OllamaEmbeddingProvider {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────── API Types ───────────────────

#[derive(Serialize)]
struct OllamaEmbeddingRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}

// ─────────────────── Trait Implementation ───────────────────

#[async_trait]
impl EmbeddingProvider for OllamaEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Embedding, AresError> {
        debug!(model = %self.model, endpoint = %self.endpoint, "Generating Ollama embedding");

        let url = format!("{}/api/embeddings", self.endpoint);
        let request = OllamaEmbeddingRequest {
            model: self.model.clone(),
            prompt: text.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                AresError::Database(format!(
                    "Ollama API request failed ({}): {e}",
                    self.endpoint
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!(status = %status, error = %body, "Ollama API error");
            return Err(AresError::Database(format!(
                "Ollama API error ({status}): {body}"
            )));
        }

        let resp: OllamaEmbeddingResponse = response.json().await.map_err(|e| {
            AresError::Serialization(format!("Failed to parse Ollama response: {e}"))
        })?;

        Ok(Embedding::new(resp.embedding, &self.model))
    }

    fn metadata(&self) -> EmbeddingMetadata {
        EmbeddingMetadata {
            provider: "ollama".to_string(),
            model: self.model.clone(),
            dimensions: self.dimensions,
            embedding_version: 1,
        }
    }
}
