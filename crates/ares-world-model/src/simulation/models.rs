use ares_core::SimulationId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Result of simulating a scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub id: SimulationId,
    pub scenario_id: ares_core::ScenarioId,
    pub task_duration_secs: f64,
    pub total_cost: f64,
    pub success_probability: f64,
    pub agent_utilization: f64,
    pub memory_usage_estimate: f64,
    pub risk_score: f64,
    pub simulated_at: DateTime<Utc>,
}

impl SimulationResult {
    /// Whether this simulation predicts likely success (>= 50%).
    pub fn is_likely_success(&self) -> bool {
        self.success_probability >= 0.5
    }

    /// Cost efficiency: success probability per cost unit.
    pub fn cost_efficiency(&self) -> f64 {
        if self.total_cost <= 0.0 {
            return self.success_probability;
        }
        self.success_probability / self.total_cost
    }

    /// Time efficiency: success probability per hour.
    pub fn time_efficiency(&self) -> f64 {
        let hours = self.task_duration_secs / 3600.0;
        if hours <= 0.0 {
            return self.success_probability;
        }
        self.success_probability / hours
    }
}

/// Configuration for running a simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Number of deterministic passes (for averaging with perturbation).
    pub iterations: u32,
    /// Required confidence level (0.0..=1.0).
    pub confidence_level: f64,
    /// Maximum time horizon in seconds.
    pub time_horizon_secs: f64,
    /// Historical success rate to blend with simulation estimate.
    pub historical_success_rate: Option<f64>,
    /// Number of historical samples backing the success rate.
    pub historical_sample_count: u32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            iterations: 1,
            confidence_level: 0.8,
            time_horizon_secs: 86400.0, // 24 hours
            historical_success_rate: None,
            historical_sample_count: 0,
        }
    }
}

/// Summary statistics from a batch of simulations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationSummary {
    pub scenario_count: usize,
    pub avg_success_probability: f64,
    pub avg_cost: f64,
    pub avg_duration_secs: f64,
    pub min_cost: f64,
    pub max_cost: f64,
    pub min_duration_secs: f64,
    pub max_duration_secs: f64,
    pub total_simulation_ms: u64,
}

impl SimulationSummary {
    /// Build a summary from a list of simulation results.
    pub fn from_results(results: &[SimulationResult], elapsed_ms: u64) -> Self {
        if results.is_empty() {
            return Self {
                scenario_count: 0,
                avg_success_probability: 0.0,
                avg_cost: 0.0,
                avg_duration_secs: 0.0,
                min_cost: 0.0,
                max_cost: 0.0,
                min_duration_secs: 0.0,
                max_duration_secs: 0.0,
                total_simulation_ms: elapsed_ms,
            };
        }

        let n = results.len() as f64;
        Self {
            scenario_count: results.len(),
            avg_success_probability: results.iter().map(|r| r.success_probability).sum::<f64>() / n,
            avg_cost: results.iter().map(|r| r.total_cost).sum::<f64>() / n,
            avg_duration_secs: results.iter().map(|r| r.task_duration_secs).sum::<f64>() / n,
            min_cost: results
                .iter()
                .map(|r| r.total_cost)
                .fold(f64::INFINITY, f64::min),
            max_cost: results
                .iter()
                .map(|r| r.total_cost)
                .fold(f64::NEG_INFINITY, f64::max),
            min_duration_secs: results
                .iter()
                .map(|r| r.task_duration_secs)
                .fold(f64::INFINITY, f64::min),
            max_duration_secs: results
                .iter()
                .map(|r| r.task_duration_secs)
                .fold(f64::NEG_INFINITY, f64::max),
            total_simulation_ms: elapsed_ms,
        }
    }
}
