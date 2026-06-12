use serde::{Deserialize, Serialize};

/// Agent reputation metrics, updated via EMA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReputation {
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub reliability: f64,
    pub cost_efficiency: f64,
    pub quality_score: f64,
    pub task_count: u64,
    pub updated_at: i64,
}

impl AgentReputation {
    pub fn new() -> Self {
        Self {
            success_rate: 0.5,
            avg_latency_ms: 0.0,
            reliability: 1.0,
            cost_efficiency: 0.5,
            quality_score: 0.5,
            task_count: 0,
            updated_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Composite reputation score (0.0 to 1.0).
    pub fn composite_score(&self) -> f64 {
        let weights = [0.30, 0.10, 0.20, 0.15, 0.25]; // success, latency, reliability, cost, quality
        let latency_score = if self.avg_latency_ms > 0.0 {
            (1.0 - (self.avg_latency_ms / 60000.0).min(1.0)).max(0.0) // Inverse: lower is better
        } else {
            0.5
        };
        let scores = [
            self.success_rate,
            latency_score,
            self.reliability,
            self.cost_efficiency,
            self.quality_score,
        ];

        scores.iter().zip(weights.iter()).map(|(s, w)| s * w).sum()
    }
}

impl Default for AgentReputation {
    fn default() -> Self {
        Self::new()
    }
}

/// Outcome data for updating reputation.
#[derive(Debug, Clone)]
pub struct TaskOutcome {
    pub success: bool,
    pub latency_ms: f64,
    pub cost: f64,
    pub quality: f64,
}
