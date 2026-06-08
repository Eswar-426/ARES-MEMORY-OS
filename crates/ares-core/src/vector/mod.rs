//! Vector intelligence module — traits, types, and similarity computation.
//!
//! This module provides the core abstractions for ARES's semantic memory engine:
//!
//! - **Traits** (`EmbeddingProvider`, `VectorRepository`) — dependency-inversion
//!   points that keep retrieval logic decoupled from concrete providers/stores.
//! - **Types** (`Embedding`, `StoredEmbedding`, `EmbeddingMetadata`,
//!   `RetrievalDiagnostics`) — the data structures that flow through the system.
//! - **Similarity** (`cosine_similarity`, `normalize_vector`) — numerically
//!   stable vector math used by the retrieval pipeline.
//!
//! Provider implementations live in `ares-embeddings`.
//! Storage implementations live in `ares-store`.

pub mod similarity;
pub mod traits;
pub mod types;

// Re-export the most commonly used items at module level.
pub use similarity::{cosine_similarity, normalize_vector, SimilarityResult};
pub use traits::{EmbeddingProvider, VectorRepository};
pub use types::{Embedding, EmbeddingMetadata, RetrievalDiagnostics, StoredEmbedding};
