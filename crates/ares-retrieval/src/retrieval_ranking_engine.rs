use crate::models::{RankingWeights, RetrievalResult};
use ares_core::types::node::GraphNode;

pub struct RetrievalRankingEngine {
    weights: RankingWeights,
}

impl RetrievalRankingEngine {
    pub fn new(weights: RankingWeights) -> Self {
        Self { weights }
    }

    /// Ranks a list of nodes based on deterministic graph scoring factors
    pub fn rank_nodes(&self, nodes: &[GraphNode]) -> Vec<RetrievalResult> {
        let mut results: Vec<RetrievalResult> = nodes
            .iter()
            .map(|n| {
                let score = self.calculate_node_score(n);
                RetrievalResult {
                    node: n.clone(),
                    score,
                }
            })
            .collect();

        // Sort descending by score
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    fn calculate_node_score(&self, node: &GraphNode) -> f32 {
        let mut total = 0.0;

        // Authority factor: presence of explicit owner
        if let Some(owners) = node.properties.get("owners") {
            if owners.as_array().is_some_and(|arr| !arr.is_empty()) {
                total += self.weights.authority;
            }
        }

        // Traceability factor: just a mock representation.
        // Real implementation would look at edge count in a memory-cached graph view.
        if node.properties.get("traceability_score").is_some() {
            total += self.weights.traceability;
        }

        // Governance factor: approvals
        if let Some(approvers) = node.properties.get("approvers") {
            if approvers.as_array().is_some_and(|arr| !arr.is_empty()) {
                total += self.weights.governance;
            }
        }

        // Freshness factor: based on updated_at
        let now = ares_core::types::event::now_micros();
        // Just a simple linear decay over a year for example purposes
        let age_micros = now.saturating_sub(node.updated_at);
        let age_days = (age_micros / (1_000_000 * 60 * 60 * 24)) as f32;
        let freshness_mult = (1.0 - (age_days / 365.0)).clamp(0.0, 1.0);
        total += self.weights.freshness * freshness_mult;

        // Completeness factor
        if node.properties.get("missing_links").is_none() {
            total += self.weights.completeness;
        }

        total
    }
}
