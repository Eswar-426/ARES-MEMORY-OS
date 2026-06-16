use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetrics {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub node_type_counts: HashMap<String, usize>,
    pub relationship_type_counts: HashMap<String, usize>,
    pub orphan_nodes: usize,
    pub largest_connected_component: usize,
    pub average_degree: f64,
    pub graph_density: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraphMetrics {
    pub call_edges: usize,
    pub dependency_edges: usize,
    pub implementation_edges: usize,
    pub resolved_symbols: usize,
    pub unresolved_symbols: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEvolutionSnapshot {
    pub timestamp: String,
    pub nodes: usize,
    pub edges: usize,
    pub largest_component: usize,
}
