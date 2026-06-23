use super::strategy::RankingStrategy;
use ares_core::GraphNode;

pub struct RecencyScorer {
    now: i64,
}

impl RecencyScorer {
    pub fn new(now: i64) -> Self {
        Self { now }
    }
}

impl RankingStrategy for RecencyScorer {
    fn score(&self, node: &GraphNode, _distance: usize) -> f64 {
        // Recency score based on created_at or updated_at (if GraphNode had it)
        // For simplicity, GraphNode has created_at
        let age_micros = self.now.saturating_sub(node.created_at);
        let age_days = (age_micros as f64) / 1_000_000.0 / 86400.0;

        // Exponential decay based on age in days. Half-life of ~30 days.
        let half_life = 30.0;
        let decay = std::f64::consts::E.powf(-0.693 * age_days / half_life);

        decay.clamp(0.0, 1.0)
    }
}
