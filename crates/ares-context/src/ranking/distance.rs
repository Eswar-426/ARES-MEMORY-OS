use super::strategy::RankingStrategy;
use ares_core::GraphNode;

pub struct DistanceScorer;

impl Default for DistanceScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl DistanceScorer {
    pub fn new() -> Self {
        Self
    }
}

impl RankingStrategy for DistanceScorer {
    fn score(&self, _node: &GraphNode, distance: usize) -> f64 {
        // Base score decreases exponentially with distance
        // depth 0: 1.0, depth 1: 0.5, depth 2: 0.25, etc.
        1.0 / (2.0_f64.powi(distance as i32))
    }
}
