use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapMetrics {
    pub gaps_before: usize,
    pub gaps_after: usize,
    pub candidates_proposed: usize,
}

impl BootstrapMetrics {
    pub fn new(gaps_before: usize, gaps_after: usize, candidates_proposed: usize) -> Self {
        Self {
            gaps_before,
            gaps_after,
            candidates_proposed,
        }
    }

    pub fn gap_closure_rate(&self) -> f64 {
        if self.gaps_before == 0 {
            return 100.0;
        }
        let closed = self.gaps_before.saturating_sub(self.gaps_after);
        (closed as f64 / self.gaps_before as f64) * 100.0
    }
}
