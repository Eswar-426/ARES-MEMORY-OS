//! ares-embeddings — Embedding provider implementations.
//!
//! This crate houses the concrete implementations of `ares_core::EmbeddingProvider`.
//! It is separated from `ares-core` to keep the core crate dependency-light
//! (no HTTP clients, no network I/O).
//!
//! ## Providers
//!
//! | Provider | Feature Flag | Default Model |
//! |----------|-------------|---------------|
//! | Mock     | *(always)*  | mock-128d     |
//! | OpenAI   | `openai`    | text-embedding-3-small |
//! | Ollama   | `ollama`    | nomic-embed-text |

pub mod providers;

pub use providers::mock::MockEmbeddingProvider;

#[cfg(feature = "openai")]
pub use providers::openai::OpenAIEmbeddingProvider;

#[cfg(feature = "ollama")]
pub use providers::ollama::OllamaEmbeddingProvider;
