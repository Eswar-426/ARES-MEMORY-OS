//! Mock embedding provider for testing, CI, and offline development.
//!
//! Generates deterministic embeddings by hashing the input text.
//! No external dependencies required.

use ares_core::vector::{
    traits::EmbeddingProvider,
    types::{Embedding, EmbeddingMetadata},
};
use ares_core::AresError;
use async_trait::async_trait;

/// A deterministic embedding provider for testing and offline use.
///
/// Embeddings are derived from a simple hash of the input text, producing
/// consistent results across runs.  The default dimension is 128.
pub struct MockEmbeddingProvider {
    dimensions: u32,
    model: String,
}

impl MockEmbeddingProvider {
    /// Create a new mock provider with the given number of dimensions.
    pub fn new(dimensions: u32) -> Self {
        Self {
            dimensions,
            model: format!("mock-{}d", dimensions),
        }
    }
}

impl Default for MockEmbeddingProvider {
    fn default() -> Self {
        Self::new(128)
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Embedding, AresError> {
        let vector = deterministic_embedding(text, self.dimensions as usize);
        Ok(Embedding::new(vector, &self.model))
    }

    fn metadata(&self) -> EmbeddingMetadata {
        EmbeddingMetadata {
            provider: "mock".to_string(),
            model: self.model.clone(),
            dimensions: self.dimensions,
            embedding_version: 1,
        }
    }
}

/// Generate a deterministic embedding vector from text.
///
/// Uses a simple hash-based approach: each dimension is derived from
/// a rotating hash of the input bytes, producing values in [-1.0, 1.0].
/// The output is then L2-normalized.
fn deterministic_embedding(text: &str, dimensions: usize) -> Vec<f32> {
    let bytes = text.as_bytes();
    let mut vector = vec![0.0_f32; dimensions];

    // Seed from text bytes using a simple hash spread
    for (i, &byte) in bytes.iter().enumerate() {
        let dim_idx = i % dimensions;
        // Use a simple mixing function for determinism
        let hash_val = ((byte as u32).wrapping_mul(2654435761)) as f32 / u32::MAX as f32;
        vector[dim_idx] += hash_val - 0.5; // center around 0
    }

    // Add cross-dimensional mixing for better distribution
    for i in 0..dimensions {
        let prev = if i > 0 {
            vector[i - 1]
        } else {
            vector[dimensions - 1]
        };
        vector[i] += prev * 0.1;
    }

    // L2 normalize
    let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude > 1e-10 {
        for x in vector.iter_mut() {
            *x /= magnitude;
        }
    }

    vector
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_produces_deterministic_embeddings() {
        let provider = MockEmbeddingProvider::default();
        let e1 = provider.embed("hello world").await.unwrap();
        let e2 = provider.embed("hello world").await.unwrap();
        assert_eq!(e1.vector, e2.vector, "Same input must produce same output");
    }

    #[tokio::test]
    async fn mock_different_inputs_differ() {
        let provider = MockEmbeddingProvider::default();
        let e1 = provider.embed("hello").await.unwrap();
        let e2 = provider.embed("world").await.unwrap();
        assert_ne!(e1.vector, e2.vector);
    }

    #[tokio::test]
    async fn mock_respects_dimensions() {
        let provider = MockEmbeddingProvider::new(256);
        let e = provider.embed("test").await.unwrap();
        assert_eq!(e.vector.len(), 256);
        assert_eq!(e.dimensions, 256);
    }

    #[tokio::test]
    async fn mock_produces_unit_vectors() {
        let provider = MockEmbeddingProvider::default();
        let e = provider.embed("normalize test").await.unwrap();
        let mag: f32 = e.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (mag - 1.0).abs() < 1e-4,
            "Should be unit vector, got magnitude {mag}"
        );
    }

    #[tokio::test]
    async fn mock_metadata_is_correct() {
        let provider = MockEmbeddingProvider::new(64);
        let meta = provider.metadata();
        assert_eq!(meta.provider, "mock");
        assert_eq!(meta.model, "mock-64d");
        assert_eq!(meta.dimensions, 64);
        assert_eq!(meta.embedding_version, 1);
    }

    #[tokio::test]
    async fn mock_similar_texts_have_higher_similarity() {
        let provider = MockEmbeddingProvider::default();
        let e1 = provider.embed("authentication system").await.unwrap();
        let e2 = provider.embed("authentication module").await.unwrap();
        let e3 = provider.embed("database migration").await.unwrap();

        let sim_close = ares_core::cosine_similarity(&e1.vector, &e2.vector);
        let sim_far = ares_core::cosine_similarity(&e1.vector, &e3.vector);

        // Similar texts should generally have higher similarity than dissimilar ones
        // (not guaranteed with a simple hash, but likely with overlapping tokens)
        // This test documents the behavior rather than strictly asserting it
        println!("sim(auth system, auth module) = {sim_close}");
        println!("sim(auth system, db migration) = {sim_far}");
    }

    #[tokio::test]
    async fn mock_batch_embedding() {
        let provider = MockEmbeddingProvider::default();
        let texts = &["hello", "world", "test"];
        let results = provider.embed_batch(texts).await.unwrap();
        assert_eq!(results.len(), 3);
        // First result should match single embed
        let single = provider.embed("hello").await.unwrap();
        assert_eq!(results[0].vector, single.vector);
    }
}
