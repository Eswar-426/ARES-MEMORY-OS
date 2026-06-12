use super::models::{HistoricalMission, SimilarityMatch};

/// Historical similarity engine — finds past missions similar to a new goal.
///
/// Uses keyword overlap and structural similarity for matching.
/// No LLM dependency — pure deterministic text matching.
pub struct SimilarityEngine;

impl Default for SimilarityEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SimilarityEngine {
    pub fn new() -> Self {
        Self
    }

    /// Find missions similar to the given goal.
    ///
    /// Returns matches sorted by similarity score (highest first),
    /// limited to `max_results`.
    pub fn find_similar(
        &self,
        goal_title: &str,
        goal_keywords: &[String],
        history: &[HistoricalMission],
        max_results: usize,
    ) -> Vec<SimilarityMatch> {
        let goal_words = self.extract_words(goal_title);

        let mut matches: Vec<SimilarityMatch> = history
            .iter()
            .map(|mission| {
                let score = self.calculate_similarity(&goal_words, goal_keywords, mission);
                let matching_keywords =
                    self.find_matching_keywords(goal_keywords, &mission.keywords);
                SimilarityMatch {
                    mission: mission.clone(),
                    similarity_score: score,
                    matching_keywords,
                }
            })
            .filter(|m| m.similarity_score > 0.05) // filter out noise
            .collect();

        // Sort by similarity descending
        matches.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        matches.truncate(max_results);
        matches
    }

    /// Calculate similarity between a goal and a historical mission.
    fn calculate_similarity(
        &self,
        goal_words: &[String],
        goal_keywords: &[String],
        mission: &HistoricalMission,
    ) -> f64 {
        let mission_words = self.extract_words(&mission.title);

        // 1. Title word overlap (Jaccard similarity)
        let title_sim = self.jaccard_similarity(goal_words, &mission_words);

        // 2. Keyword overlap
        let keyword_sim = self.jaccard_similarity(goal_keywords, &mission.keywords);

        // 3. Structural similarity (step count, agent count — fuzzy match)
        // This is a lightweight comparison; more data gives better results
        let _structural_sim = 0.0; // Placeholder — enhanced when fed real data

        // Weighted combination
        let score = title_sim * 0.5 + keyword_sim * 0.5;
        score.clamp(0.0, 1.0)
    }

    /// Jaccard similarity: |A ∩ B| / |A ∪ B|
    fn jaccard_similarity(&self, a: &[String], b: &[String]) -> f64 {
        if a.is_empty() && b.is_empty() {
            return 0.0;
        }

        let set_a: std::collections::HashSet<&str> = a.iter().map(|s| s.as_str()).collect();
        let set_b: std::collections::HashSet<&str> = b.iter().map(|s| s.as_str()).collect();

        let intersection = set_a.intersection(&set_b).count() as f64;
        let union = set_a.union(&set_b).count() as f64;

        if union == 0.0 {
            0.0
        } else {
            intersection / union
        }
    }

    /// Find keywords that appear in both lists.
    fn find_matching_keywords(&self, goal_kw: &[String], mission_kw: &[String]) -> Vec<String> {
        let mission_set: std::collections::HashSet<&str> =
            mission_kw.iter().map(|s| s.as_str()).collect();

        goal_kw
            .iter()
            .filter(|kw| mission_set.contains(kw.as_str()))
            .cloned()
            .collect()
    }

    /// Extract normalized words from a title (lowercase, no stopwords).
    fn extract_words(&self, text: &str) -> Vec<String> {
        let stopwords = [
            "a", "an", "the", "is", "it", "to", "for", "of", "in", "on", "at", "by", "with", "and",
            "or", "but", "from", "as", "that", "this", "be", "are", "was", "were", "been", "being",
        ];

        text.to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 1)
            .filter(|w| !stopwords.contains(w))
            .map(|w| {
                w.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
            })
            .filter(|w| !w.is_empty())
            .collect()
    }

    /// Extract keywords from a goal title for similarity matching.
    pub fn extract_keywords(&self, text: &str) -> Vec<String> {
        self.extract_words(text)
    }

    /// Compute aggregate statistics from similar missions.
    pub fn aggregate_stats(&self, matches: &[SimilarityMatch]) -> SimilarityStats {
        if matches.is_empty() {
            return SimilarityStats::default();
        }

        let total = matches.len() as f64;
        let total_weight: f64 = matches.iter().map(|m| m.similarity_score).sum();

        let success_count = matches.iter().filter(|m| m.mission.success).count();

        let weighted_cost: f64 = if total_weight > 0.0 {
            matches
                .iter()
                .map(|m| m.mission.cost * m.similarity_score)
                .sum::<f64>()
                / total_weight
        } else {
            matches.iter().map(|m| m.mission.cost).sum::<f64>() / total
        };

        let weighted_duration: f64 = if total_weight > 0.0 {
            matches
                .iter()
                .map(|m| m.mission.duration_secs * m.similarity_score)
                .sum::<f64>()
                / total_weight
        } else {
            matches.iter().map(|m| m.mission.duration_secs).sum::<f64>() / total
        };

        SimilarityStats {
            match_count: matches.len() as u32,
            success_rate: success_count as f64 / total,
            avg_cost: weighted_cost,
            avg_duration_secs: weighted_duration,
            avg_similarity: total_weight / total,
        }
    }
}

/// Aggregate statistics from similarity matches.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SimilarityStats {
    pub match_count: u32,
    pub success_rate: f64,
    pub avg_cost: f64,
    pub avg_duration_secs: f64,
    pub avg_similarity: f64,
}
