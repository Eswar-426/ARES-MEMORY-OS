//! ares-extractor — Autonomous Knowledge Extraction Engine for ARES MemoryOS.
//!
//! Takes git commits and extracts structured project knowledge (decisions,
//! architecture changes, bugs, experiments) using an LLM provider. Extracted
//! knowledge is stored as `KnowledgeCandidate` records and persisted into the
//! ARES memory store when they meet the configured confidence threshold.

pub mod engine;
pub mod git;
pub mod provider;

pub use engine::ExtractionEngine;
pub use provider::{ExtractorProvider, MockExtractorProvider};
