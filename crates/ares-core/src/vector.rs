use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::AresError;

/// A vector representation of text/code.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Embedding {
    pub vector: Vec<f32>,
    pub model: String,
}

/// Interface for generating embeddings.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn generate_embedding(&self, text: &str) -> Result<Embedding, AresError>;
    async fn generate_embeddings(&self, texts: &[&str]) -> Result<Vec<Embedding>, AresError>;
}

/// Interface for vector search operations.
#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn store_embedding(&self, id: &str, embedding: &Embedding) -> Result<(), AresError>;
    async fn search(
        &self,
        query_embedding: &Embedding,
        limit: usize,
    ) -> Result<Vec<(String, f32)>, AresError>;
}
