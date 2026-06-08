//! Embedding and vector-related type definitions.

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────
// Embedding — the vector output from a provider
// ─────────────────────────────────────────────────────────────────

/// A vector representation of text/code, produced by an `EmbeddingProvider`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Embedding {
    /// The dense vector.
    pub vector: Vec<f32>,
    /// The model that produced this embedding (e.g. "text-embedding-3-small").
    pub model: String,
    /// Number of dimensions (== vector.len(), stored for validation).
    pub dimensions: u32,
}

impl Embedding {
    /// Create a new `Embedding`, deriving `dimensions` from the vector length.
    pub fn new(vector: Vec<f32>, model: impl Into<String>) -> Self {
        let dimensions = vector.len() as u32;
        Self {
            vector,
            model: model.into(),
            dimensions,
        }
    }
}

// ─────────────────────────────────────────────────────────────────
// Embedding Metadata — provider + model versioning
// ─────────────────────────────────────────────────────────────────

/// Metadata describing *how* an embedding was generated.
///
/// Stored alongside every embedding to prevent mixing vectors from different
/// models and to support future re-indexing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbeddingMetadata {
    /// Provider name (e.g. "openai", "ollama", "mock").
    pub provider: String,
    /// Model identifier (e.g. "text-embedding-3-small").
    pub model: String,
    /// Number of dimensions the model produces.
    pub dimensions: u32,
    /// Monotonic version counter — bump when re-indexing with a new model.
    pub embedding_version: u32,
}

// ─────────────────────────────────────────────────────────────────
// Stored Embedding — what comes out of the VectorRepository
// ─────────────────────────────────────────────────────────────────

/// A persisted embedding with its metadata and creation timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEmbedding {
    pub memory_id: String,
    pub embedding: Vec<f32>,
    pub provider: String,
    pub model: String,
    pub dimensions: u32,
    pub embedding_version: u32,
    pub created_at: String,
}

// ─────────────────────────────────────────────────────────────────
// Retrieval Diagnostics
// ─────────────────────────────────────────────────────────────────

/// Diagnostic data collected during a semantic search operation.
///
/// Attached to every search response for observability, debugging,
/// and future reasoning-engine consumption.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RetrievalDiagnostics {
    /// Provider used for the query embedding.
    pub provider: String,
    /// Model used for the query embedding.
    pub model: String,
    /// Time to generate the query embedding (ms).
    pub embedding_latency_ms: f64,
    /// Time to perform vector search (ms).
    pub vector_search_latency_ms: f64,
    /// Time to perform keyword search (ms).
    pub keyword_search_latency_ms: f64,
    /// Time for the full ranking pipeline (ms).
    pub ranking_latency_ms: f64,
    /// Total end-to-end latency (ms).
    pub total_latency_ms: f64,
    /// Number of vector candidates considered.
    pub vector_candidates: usize,
    /// Number of keyword candidates considered.
    pub keyword_candidates: usize,
    /// Number of results after merging and deduplication.
    pub merged_candidates: usize,
    /// Number of final results returned.
    pub results_returned: usize,
}
