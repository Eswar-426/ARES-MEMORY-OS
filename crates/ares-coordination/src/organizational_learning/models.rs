use serde::{Deserialize, Serialize};

/// EMA-tracked performance for a team composition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPerformance {
    pub team_key: String,
    pub ema_success_rate: f64,
    pub ema_quality: f64,
    pub ema_cost: f64,
    pub sample_count: u64,
    pub updated_at: i64,
}

impl TeamPerformance {
    pub fn new(team_key: impl Into<String>) -> Self {
        Self {
            team_key: team_key.into(),
            ema_success_rate: 0.5,
            ema_quality: 0.5,
            ema_cost: 0.0,
            sample_count: 0,
            updated_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn composite_score(&self) -> f64 {
        self.ema_success_rate * 0.4 + self.ema_quality * 0.4 + (1.0 - self.ema_cost.min(1.0)) * 0.2
    }
}

/// EMA-tracked performance for an agent pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairPerformance {
    pub pair_key: String,
    pub ema_synergy: f64,
    pub ema_quality: f64,
    pub sample_count: u64,
    pub updated_at: i64,
}

impl PairPerformance {
    pub fn new(pair_key: impl Into<String>) -> Self {
        Self {
            pair_key: pair_key.into(),
            ema_synergy: 0.5,
            ema_quality: 0.5,
            sample_count: 0,
            updated_at: chrono::Utc::now().timestamp(),
        }
    }
}

/// EMA-tracked performance for a workflow structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPerformance {
    pub workflow_key: String,
    pub ema_success_rate: f64,
    pub ema_throughput: f64,
    pub sample_count: u64,
    pub updated_at: i64,
}

impl WorkflowPerformance {
    pub fn new(workflow_key: impl Into<String>) -> Self {
        Self {
            workflow_key: workflow_key.into(),
            ema_success_rate: 0.5,
            ema_throughput: 0.0,
            sample_count: 0,
            updated_at: chrono::Utc::now().timestamp(),
        }
    }
}
