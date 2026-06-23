//! ares-context-generator — Produces portable, AI-context-window-friendly output.
//!
//! Takes a ProjectSnapshot and transforms it into structured text that can be
//! pasted into any AI model's context window (ChatGPT, Claude, Gemini, Cursor)
//! to restore full project understanding without re-explanation.

#![allow(dead_code)]
pub mod compressor;
pub mod generator;
pub mod summarizer;
pub mod templates;
pub mod types;

pub use compressor::ContextCompressor;
pub use generator::ContextGenerator;
pub use summarizer::MemorySummarizer;
pub use types::PortableContext;
