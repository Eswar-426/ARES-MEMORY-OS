use ares_core::types::memory::Memory;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedResult {
    pub memory: Memory,
    pub score: f32,
    pub semantic_score: f32,
    pub recency_score: f32,
    pub importance_score: f32,
    pub match_reason: String,
}
