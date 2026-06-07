use ares_core::{ImportanceLevel, Memory, MemoryId};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryScore {
    pub relevance: f32,
    pub confidence: f32,
    pub importance: f32,
    pub recency: f32,
    pub frequency: f32,
    pub final_score: f32,
}

pub struct MemoryRankingEngine;

impl MemoryRankingEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MemoryRankingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryRankingEngine {
    pub fn rank_memories(
        &self,
        candidate_memories: &[Memory],
        relevance_scores: &[(MemoryId, f32)],
        access_counts: &[(MemoryId, u32)],
    ) -> Vec<(Memory, MemoryScore)> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;

        let mut results = Vec::with_capacity(candidate_memories.len());

        for mem in candidate_memories {
            let relevance = relevance_scores
                .iter()
                .find(|(id, _)| id == &mem.id)
                .map(|(_, score)| *score)
                .unwrap_or(0.0);

            let frequency = access_counts
                .iter()
                .find(|(id, _)| id == &mem.id)
                .map(|(_, count)| *count as f32)
                .unwrap_or(0.0);

            let score = self.calculate_score(mem, relevance, frequency, now);
            results.push((mem.clone(), score));
        }

        // Sort by final_score descending
        results.sort_by(|a, b| {
            b.1.final_score
                .partial_cmp(&a.1.final_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    fn calculate_score(
        &self,
        memory: &Memory,
        relevance: f32,
        frequency: f32,
        now: i64,
    ) -> MemoryScore {
        let age_micros = now.saturating_sub(memory.updated_at).max(0);
        let age_days = age_micros as f32 / (1_000_000.0 * 60.0 * 60.0 * 24.0);

        // Importance base score
        let importance_base = match memory.importance {
            ImportanceLevel::Critical => 1.0,
            ImportanceLevel::High => 0.8,
            ImportanceLevel::Medium => 0.5,
            ImportanceLevel::Low => 0.2,
        };

        // Recency decay: half-life based on importance
        let half_life_days = match memory.importance {
            ImportanceLevel::Critical => 365.0, // Critical decays very slowly
            ImportanceLevel::High => 90.0,
            ImportanceLevel::Medium => 30.0,
            ImportanceLevel::Low => 7.0,
        };

        let recency = 2.0_f32.powf(-age_days / half_life_days);

        // Frequency scaling
        let frequency_score = (frequency / 10.0).min(1.0); // Caps at 10 accesses for max frequency score

        // Weights
        let w_relevance = 0.4;
        let w_confidence = 0.1;
        let w_importance = 0.2;
        let w_recency = 0.2;
        let w_frequency = 0.1;

        let final_score = (relevance * w_relevance)
            + (memory.confidence * w_confidence)
            + (importance_base * w_importance)
            + (recency * w_recency)
            + (frequency_score * w_frequency);

        MemoryScore {
            relevance,
            confidence: memory.confidence,
            importance: importance_base,
            recency,
            frequency: frequency_score,
            final_score,
        }
    }
}
