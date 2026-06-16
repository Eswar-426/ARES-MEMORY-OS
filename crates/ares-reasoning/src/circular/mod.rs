use crate::graph::ReasoningGraph;
use crate::models::CircularDependency;

pub struct CircularDependencyAnalyzer;

impl CircularDependencyAnalyzer {
    pub fn analyze(graph: &ReasoningGraph) -> Vec<CircularDependency> {
        let (pgraph, node_indices) = graph.to_petgraph();
        let mut idx_to_id = std::collections::HashMap::new();
        for (id, idx) in &node_indices {
            idx_to_id.insert(*idx, id.clone());
        }

        let sccs = petgraph::algo::tarjan_scc(&pgraph);
        let mut cycles = Vec::new();

        for scc in sccs {
            if scc.len() >= 2 {
                let cycle_length = scc.len();
                // A 2-node cycle = severe (100). A 12-node cycle = moderate (16.6)
                let severity = (200.0 / cycle_length as f64).min(100.0).max(0.0);

                let nodes = scc
                    .iter()
                    .filter_map(|idx| idx_to_id.get(idx).cloned())
                    .collect();
                cycles.push(CircularDependency {
                    nodes,
                    cycle_length,
                    severity,
                });
            }
        }

        cycles.sort_by(|a, b| {
            b.severity
                .partial_cmp(&a.severity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        cycles
    }
}
