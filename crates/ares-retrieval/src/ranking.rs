use crate::modes::RetrievalMode;
use crate::types::RankedResult;
use ares_core::types::memory::{ImportanceLevel, Memory};
use chrono::Utc;

pub struct RetrievalRanker {
    mode: RetrievalMode,
}

impl RetrievalRanker {
    pub fn new(mode: RetrievalMode) -> Self {
        Self { mode }
    }

    pub fn rank(&self, candidates: &[(Memory, f32)]) -> Vec<RankedResult> {
        let now = Utc::now().timestamp_micros();

        let mut results: Vec<RankedResult> = candidates
            .iter()
            .map(|(memory, semantic_score)| {
                let age_seconds = ((now - memory.created_at) as f32) / 1_000_000.0;
                let recency_score = self.calculate_recency_score(age_seconds.max(0.0));
                let importance_score = match memory.importance {
                    ImportanceLevel::Critical => 1.0,
                    ImportanceLevel::High => 0.8,
                    ImportanceLevel::Medium => 0.5,
                    ImportanceLevel::Low => 0.2,
                };

                let weights = self.get_weights();
                let final_score = (semantic_score * weights.0)
                    + (recency_score * weights.1)
                    + (importance_score * weights.2);

                RankedResult {
                    memory: memory.clone(),
                    score: final_score,
                    semantic_score: *semantic_score,
                    recency_score,
                    importance_score,
                    match_reason: self.generate_reason(
                        semantic_score,
                        recency_score,
                        importance_score,
                    ),
                }
            })
            .collect();

        // Sort descending by final score
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    fn calculate_recency_score(&self, age_seconds: f32) -> f32 {
        let half_life = match self.mode {
            RetrievalMode::RecentContext => 86400.0 * 3.0, // 3 days
            _ => 86400.0 * 30.0,                           // 30 days
        };

        (0.5_f32).powf(age_seconds / half_life).clamp(0.0, 1.0)
    }

    /// Returns (semantic_weight, recency_weight, importance_weight)
    fn get_weights(&self) -> (f32, f32, f32) {
        match self.mode {
            RetrievalMode::General => (0.5, 0.2, 0.3),
            RetrievalMode::RecentContext => (0.3, 0.6, 0.1),
            RetrievalMode::ProjectSummary => (0.4, 0.1, 0.5),
            RetrievalMode::DecisionHistory
            | RetrievalMode::BugHistory
            | RetrievalMode::FeatureHistory => (0.7, 0.1, 0.2),
            RetrievalMode::ArchitectureContext => (0.6, 0.0, 0.4),
        }
    }

    fn generate_reason(&self, semantic: &f32, recency: f32, importance: f32) -> String {
        let mut reasons = Vec::new();
        if *semantic > 0.8 {
            reasons.push("Highly relevant to query");
        }
        if recency > 0.8 {
            reasons.push("Very recent");
        }
        if importance > 0.8 {
            reasons.push("High importance");
        }
        if reasons.is_empty() {
            "General match".to_string()
        } else {
            reasons.join(", ")
        }
    }
}
