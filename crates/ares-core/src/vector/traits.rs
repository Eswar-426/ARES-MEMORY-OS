//! Trait definitions for embedding generation and vector storage.
//!
//! These traits live in `ares-core` so that higher-level crates can depend on
//! abstractions rather than concrete implementations.  Provider implementations
//! live in `ares-embeddings`; storage implementations live in `ares-store`.

use async_trait::async_trait;

use super::similarity::SimilarityResult;
use super::types::{Embedding, EmbeddingMetadata, StoredEmbedding};
use crate::AresError;

// ─────────────────────────────────────────────────────────────────
// Embedding Provider
// ─────────────────────────────────────────────────────────────────

/// Provider-agnostic interface for generating text embeddings.
///
/// Implementations must be `Send + Sync` to allow sharing across async tasks.
/// The trait is object-safe so it can be used as `Arc<dyn EmbeddingProvider>`.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate an embedding for a single text input.
    async fn embed(&self, text: &str) -> Result<Embedding, AresError>;

    /// Generate embeddings for multiple texts in a single batch.
    ///
    /// Default implementation calls `embed` sequentially.  Providers that
    /// support batch APIs (e.g. OpenAI) should override this for efficiency.
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Embedding>, AresError> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }

    /// Return metadata describing this provider's configuration.
    fn metadata(&self) -> EmbeddingMetadata;
}

// ─────────────────────────────────────────────────────────────────
// Vector Repository
// ─────────────────────────────────────────────────────────────────

/// Storage-agnostic interface for persisting and querying embeddings.
///
/// Current implementation: `SqliteVectorRepository` in `ares-store`.
/// Future implementations: Qdrant, pgvector, LanceDB, FAISS, etc.
pub trait VectorRepository: Send + Sync {
    /// Insert or replace the embedding for a given memory.
    fn upsert_embedding(
        &self,
        memory_id: &str,
        embedding: &[f32],
        metadata: &EmbeddingMetadata,
    ) -> Result<(), AresError>;

    /// Retrieve a stored embedding by memory ID.
    fn get_embedding(&self, memory_id: &str) -> Result<Option<StoredEmbedding>, AresError>;

    /// Delete the embedding for a given memory.
    fn delete_embedding(&self, memory_id: &str) -> Result<(), AresError>;

    /// Find the most similar embeddings to the query vector.
    ///
    /// The search MUST be filtered to only compare against embeddings generated
    /// by the same provider and model (via `metadata`) to prevent comparing
    /// incompatible vector spaces.
    /// Returns up to `limit` results sorted by descending similarity score.
    fn search_similar(
        &self,
        query_embedding: &[f32],
        metadata: &EmbeddingMetadata,
        limit: usize,
    ) -> Result<Vec<SimilarityResult>, AresError>;

    /// Return the total number of stored embeddings.
    fn count(&self) -> Result<u64, AresError>;

    /// Return all memory IDs that have stored embeddings.
    fn list_embedded_memory_ids(&self) -> Result<Vec<String>, AresError>;
}
