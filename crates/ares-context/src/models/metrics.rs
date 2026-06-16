use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextMetrics {
    pub retrieval_time_ms: u64,
    pub traversal_time_ms: u64,
    pub ranking_time_ms: u64,
    pub nodes_examined: usize,
    pub nodes_returned: usize,
    pub nodes_selected: usize,
    pub files_selected: usize,
    pub token_estimate: usize,
    pub avg_depth: f64,
    pub max_depth: usize,
    pub context_efficiency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RetrievalAudit {
    pub query: String,
    pub seed_nodes: usize,
    pub retrieved_nodes: usize,
    pub reachable_nodes: usize,
    pub retrieval_depth: usize,
    pub graph_coverage_score: f64,
    pub retrieval_latency_ms: u64,
}
