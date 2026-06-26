use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustScore {
    pub score: f32,
    pub evidence_count: usize,
    pub manual_approvals: usize,
    pub revalidation_successes: usize,
    pub contradiction_signals: usize,
    pub is_trusted: bool,
}

impl Default for TrustScore {
    fn default() -> Self {
        Self {
            score: 0.0,
            evidence_count: 0,
            manual_approvals: 0,
            revalidation_successes: 0,
            contradiction_signals: 0,
            is_trusted: false,
        }
    }
}
