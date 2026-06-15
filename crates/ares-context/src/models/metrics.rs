use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextMetrics {
    pub retrieval_time_ms: u64,
    pub traversal_time_ms: u64,
    pub ranking_time_ms: u64,
    pub nodes_examined: usize,
    pub nodes_returned: usize,
}
