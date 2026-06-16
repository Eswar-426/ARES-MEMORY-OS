use crate::graph::ReasoningGraph;
use crate::models::Bottleneck;

pub struct BottleneckAnalyzer;

impl BottleneckAnalyzer {
    pub fn analyze(graph: &ReasoningGraph) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();
        let max_nodes = graph.nodes.len() as f64;

        for id in graph.nodes.keys() {
            let in_degree = graph.incoming.get(id).map(|e| e.len()).unwrap_or(0);
            let out_degree = graph.outgoing.get(id).map(|e| e.len()).unwrap_or(0);
            let degree = in_degree + out_degree;

            let degree_score = if max_nodes > 0.0 {
                (degree as f64 / max_nodes) * 100.0
            } else {
                0.0
            };

            let betweenness = (in_degree * out_degree) as f64;
            let centrality_score = if max_nodes > 0.0 {
                // Approximate max possible betweenness for naive algorithm
                let max_betweenness = (max_nodes / 2.0) * (max_nodes / 2.0);
                (betweenness / max_betweenness.max(1.0)) * 100.0
            } else {
                0.0
            };

            let risk_score = (degree_score + centrality_score) / 2.0;

            if risk_score > 0.0 {
                bottlenecks.push(Bottleneck {
                    node_id: id.clone(),
                    degree,
                    in_degree,
                    out_degree,
                    betweenness,
                    risk_score,
                });
            }
        }

        bottlenecks.sort_by(|a, b| {
            b.risk_score
                .partial_cmp(&a.risk_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        bottlenecks
    }
}
