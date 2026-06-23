use ares_core::GraphNode;

pub struct RankedNode {
    pub node: GraphNode,
    pub score: f64,
}

pub trait RankingStrategy: Send + Sync {
    /// Score a single node given its context (e.g. distance from seed)
    fn score(&self, node: &GraphNode, distance: usize) -> f64;

    /// Sorts a vector of nodes and returns them with their scores
    fn rank(&self, mut nodes: Vec<(GraphNode, usize)>) -> Vec<RankedNode> {
        let mut ranked: Vec<RankedNode> = nodes
            .drain(..)
            .map(|(node, dist)| RankedNode {
                score: self.score(&node, dist),
                node,
            })
            .collect();

        // Sort descending
        ranked.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        ranked
    }
}
