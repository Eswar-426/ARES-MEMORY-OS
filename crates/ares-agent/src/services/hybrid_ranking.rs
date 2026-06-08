//! Hybrid ranking engine for semantic search.
//!
//! Combines multiple ranking signals (semantic similarity, keyword match,
//! importance, recency, graph connectivity) into a single final score.
//! Weights are fully configurable.

use ares_core::{ImportanceLevel, Memory, MemoryId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ─────────────────────────────────────────────────────────────────
// Configuration
// ─────────────────────────────────────────────────────────────────

/// Configurable weights for the hybrid ranking formula.
///
/// Defaults: semantic=0.40, keyword=0.25, importance=0.15, recency=0.10, graph=0.10
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridRankingConfig {
    pub semantic_weight: f32,
    pub keyword_weight: f32,
    pub importance_weight: f32,
    pub recency_weight: f32,
    pub graph_weight: f32,
}

impl Default for HybridRankingConfig {
    fn default() -> Self {
        Self {
            semantic_weight: 0.40,
            keyword_weight: 0.25,
            importance_weight: 0.15,
            recency_weight: 0.10,
            graph_weight: 0.10,
        }
    }
}

impl HybridRankingConfig {
    /// Validate that weights are non-negative.
    pub fn validate(&self) -> Result<(), String> {
        if self.semantic_weight < 0.0
            || self.keyword_weight < 0.0
            || self.importance_weight < 0.0
            || self.recency_weight < 0.0
            || self.graph_weight < 0.0
        {
            return Err("All weights must be non-negative".into());
        }
        Ok(())
    }

    /// Return the sum of all weights (for normalization).
    pub fn total_weight(&self) -> f32 {
        self.semantic_weight
            + self.keyword_weight
            + self.importance_weight
            + self.recency_weight
            + self.graph_weight
    }
}

// ─────────────────────────────────────────────────────────────────
// Result Types
// ─────────────────────────────────────────────────────────────────

/// A search result with all sub-scores and the final hybrid score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchResult {
    pub memory: Memory,
    pub semantic_score: f32,
    pub keyword_score: f32,
    pub importance_score: f32,
    pub recency_score: f32,
    pub graph_score: f32,
    pub final_score: f32,
}

// ─────────────────────────────────────────────────────────────────
// Engine
// ─────────────────────────────────────────────────────────────────

pub struct HybridRankingEngine {
    config: HybridRankingConfig,
}

impl HybridRankingEngine {
    pub fn new(config: HybridRankingConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self {
            config: HybridRankingConfig::default(),
        }
    }

    /// Rank a set of candidate memories using all available signals.
    ///
    /// # Arguments
    /// - `candidates` — deduplicated memories to rank
    /// - `semantic_scores` — memory_id → cosine similarity score (0.0–1.0)
    /// - `keyword_scores` — memory_id → FTS relevance score (normalized)
    /// - `graph_scores` — memory_id → graph connectivity score (0.0–1.0)
    pub fn rank(
        &self,
        candidates: &[Memory],
        semantic_scores: &HashMap<MemoryId, f32>,
        keyword_scores: &HashMap<MemoryId, f32>,
        graph_scores: &HashMap<MemoryId, f32>,
    ) -> Vec<SemanticSearchResult> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;

        let total_weight = self.config.total_weight();
        if total_weight < 1e-8 {
            // All weights zero — return unsorted
            return candidates
                .iter()
                .map(|m| SemanticSearchResult {
                    memory: m.clone(),
                    semantic_score: 0.0,
                    keyword_score: 0.0,
                    importance_score: 0.0,
                    recency_score: 0.0,
                    graph_score: 0.0,
                    final_score: 0.0,
                })
                .collect();
        }

        let mut results: Vec<SemanticSearchResult> = candidates
            .iter()
            .map(|memory| {
                let sem = *semantic_scores.get(&memory.id).unwrap_or(&0.0);
                let kw = *keyword_scores.get(&memory.id).unwrap_or(&0.0);
                let graph = *graph_scores.get(&memory.id).unwrap_or(&0.0);
                let importance = importance_to_score(&memory.importance);
                let recency = recency_score(memory.updated_at, now);

                let raw_score = (sem * self.config.semantic_weight)
                    + (kw * self.config.keyword_weight)
                    + (importance * self.config.importance_weight)
                    + (recency * self.config.recency_weight)
                    + (graph * self.config.graph_weight);

                // Normalize by total weight so the final score is in [0, 1]
                let final_score = raw_score / total_weight;

                SemanticSearchResult {
                    memory: memory.clone(),
                    semantic_score: sem,
                    keyword_score: kw,
                    importance_score: importance,
                    recency_score: recency,
                    graph_score: graph,
                    final_score,
                }
            })
            .collect();

        // Sort descending by final score
        results.sort_by(|a, b| {
            b.final_score
                .partial_cmp(&a.final_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Get a reference to the current config.
    pub fn config(&self) -> &HybridRankingConfig {
        &self.config
    }
}

// ─────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────

fn importance_to_score(level: &ImportanceLevel) -> f32 {
    match level {
        ImportanceLevel::Critical => 1.0,
        ImportanceLevel::High => 0.8,
        ImportanceLevel::Medium => 0.5,
        ImportanceLevel::Low => 0.2,
    }
}

/// Exponential recency decay.  Half-life = 30 days.
fn recency_score(updated_at_micros: i64, now_micros: i64) -> f32 {
    let age_micros = now_micros.saturating_sub(updated_at_micros).max(0);
    let age_days = age_micros as f64 / (1_000_000.0 * 60.0 * 60.0 * 24.0);
    let half_life = 30.0_f64;
    2.0_f64.powf(-age_days / half_life) as f32
}

// ─────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ares_core::*;

    fn make_memory(id: &str, importance: ImportanceLevel, updated_at: i64) -> Memory {
        Memory {
            id: MemoryId::from(id),
            project_id: ProjectId::from("proj_1"),
            memory_type: MemoryType::Feature,
            title: format!("Memory {id}"),
            content: serde_json::json!({}),
            status: MemoryStatus::Active,
            version: 1,
            parent_id: None,
            confidence: 1.0,
            importance,
            source: MemorySource::Human,
            ai_assisted: false,
            created_at: updated_at,
            updated_at,
            deleted_at: None,
        }
    }

    #[test]
    fn default_weights_sum_to_one() {
        let config = HybridRankingConfig::default();
        let total = config.total_weight();
        assert!(
            (total - 1.0).abs() < 1e-6,
            "Weights should sum to 1.0, got {total}"
        );
    }

    #[test]
    fn higher_semantic_score_ranks_higher() {
        let engine = HybridRankingEngine::with_default_config();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;

        let m1 = make_memory("m1", ImportanceLevel::Medium, now);
        let m2 = make_memory("m2", ImportanceLevel::Medium, now);

        let mut semantic = HashMap::new();
        semantic.insert(MemoryId::from("m1"), 0.9_f32);
        semantic.insert(MemoryId::from("m2"), 0.1_f32);

        let results = engine.rank(&[m1, m2], &semantic, &HashMap::new(), &HashMap::new());
        assert_eq!(results[0].memory.id.as_str(), "m1");
        assert!(results[0].final_score > results[1].final_score);
    }

    #[test]
    fn importance_affects_ranking() {
        let engine = HybridRankingEngine::with_default_config();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;

        let m_critical = make_memory("critical", ImportanceLevel::Critical, now);
        let m_low = make_memory("low", ImportanceLevel::Low, now);

        // No semantic or keyword scores — pure importance + recency
        let results = engine.rank(
            &[m_low, m_critical],
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert_eq!(results[0].memory.id.as_str(), "critical");
    }

    #[test]
    fn recency_decay_works() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;

        let recent = recency_score(now, now);
        assert!((recent - 1.0).abs() < 1e-4, "Just now should be ~1.0");

        // 30 days ago should be ~0.5 (half-life)
        let thirty_days_ago = now - (30 * 24 * 60 * 60 * 1_000_000_i64);
        let old = recency_score(thirty_days_ago, now);
        assert!(
            (old - 0.5).abs() < 0.05,
            "30-day-old should be ~0.5, got {old}"
        );
    }

    #[test]
    fn zero_weights_return_zero_scores() {
        let config = HybridRankingConfig {
            semantic_weight: 0.0,
            keyword_weight: 0.0,
            importance_weight: 0.0,
            recency_weight: 0.0,
            graph_weight: 0.0,
        };
        let engine = HybridRankingEngine::new(config);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;

        let m = make_memory("m1", ImportanceLevel::Critical, now);
        let results = engine.rank(&[m], &HashMap::new(), &HashMap::new(), &HashMap::new());
        assert_eq!(results[0].final_score, 0.0);
    }

    #[test]
    fn empty_candidates_return_empty() {
        let engine = HybridRankingEngine::with_default_config();
        let results = engine.rank(&[], &HashMap::new(), &HashMap::new(), &HashMap::new());
        assert!(results.is_empty());
    }

    #[test]
    fn config_validation() {
        let valid = HybridRankingConfig::default();
        assert!(valid.validate().is_ok());

        let invalid = HybridRankingConfig {
            semantic_weight: -0.1,
            ..Default::default()
        };
        assert!(invalid.validate().is_err());
    }
}
