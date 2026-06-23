use super::distance::DistanceScorer;
use super::recency::RecencyScorer;
use super::strategy::RankingStrategy;
use ares_core::GraphNode;

pub struct HybridRanker {
    distance_scorer: DistanceScorer,
    recency_scorer: RecencyScorer,
    distance_weight: f64,
    recency_weight: f64,
}

impl HybridRanker {
    pub fn new(now: i64, distance_weight: f64, recency_weight: f64) -> Self {
        Self {
            distance_scorer: DistanceScorer::new(),
            recency_scorer: RecencyScorer::new(now),
            distance_weight,
            recency_weight,
        }
    }
}

impl RankingStrategy for HybridRanker {
    fn score(&self, node: &GraphNode, distance: usize) -> f64 {
        let dist_score = self.distance_scorer.score(node, distance);
        let recency_score = self.recency_scorer.score(node, distance);

        (dist_score * self.distance_weight) + (recency_score * self.recency_weight)
    }
}
