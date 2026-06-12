use std::collections::HashMap;

use super::models::{PairPerformance, TeamPerformance, WorkflowPerformance};

const EMA_ALPHA: f64 = 0.2;

/// Engine for tracking organizational learning across teams, pairs, and workflows.
pub struct OrgLearningEngine {
    teams: HashMap<String, TeamPerformance>,
    pairs: HashMap<String, PairPerformance>,
    workflows: HashMap<String, WorkflowPerformance>,
}

impl OrgLearningEngine {
    pub fn new() -> Self {
        Self {
            teams: HashMap::new(),
            pairs: HashMap::new(),
            workflows: HashMap::new(),
        }
    }

    /// Record a team outcome.
    pub fn record_team_outcome(&mut self, team_key: &str, success: bool, quality: f64, cost: f64) {
        let perf = self
            .teams
            .entry(team_key.to_string())
            .or_insert_with(|| TeamPerformance::new(team_key));

        let success_val = if success { 1.0 } else { 0.0 };

        if perf.sample_count == 0 {
            perf.ema_success_rate = success_val;
            perf.ema_quality = quality;
            perf.ema_cost = cost;
        } else {
            perf.ema_success_rate = ema(perf.ema_success_rate, success_val);
            perf.ema_quality = ema(perf.ema_quality, quality);
            perf.ema_cost = ema(perf.ema_cost, cost);
        }
        perf.sample_count += 1;
        perf.updated_at = chrono::Utc::now().timestamp();
    }

    /// Record a pair outcome.
    pub fn record_pair_outcome(&mut self, pair_key: &str, synergy_score: f64, quality: f64) {
        let perf = self
            .pairs
            .entry(pair_key.to_string())
            .or_insert_with(|| PairPerformance::new(pair_key));

        if perf.sample_count == 0 {
            perf.ema_synergy = synergy_score;
            perf.ema_quality = quality;
        } else {
            perf.ema_synergy = ema(perf.ema_synergy, synergy_score);
            perf.ema_quality = ema(perf.ema_quality, quality);
        }
        perf.sample_count += 1;
        perf.updated_at = chrono::Utc::now().timestamp();
    }

    /// Record a workflow outcome.
    pub fn record_workflow_outcome(&mut self, workflow_key: &str, success: bool, throughput: f64) {
        let perf = self
            .workflows
            .entry(workflow_key.to_string())
            .or_insert_with(|| WorkflowPerformance::new(workflow_key));

        let success_val = if success { 1.0 } else { 0.0 };

        if perf.sample_count == 0 {
            perf.ema_success_rate = success_val;
            perf.ema_throughput = throughput;
        } else {
            perf.ema_success_rate = ema(perf.ema_success_rate, success_val);
            perf.ema_throughput = ema(perf.ema_throughput, throughput);
        }
        perf.sample_count += 1;
        perf.updated_at = chrono::Utc::now().timestamp();
    }

    /// Recommend the best team for a task type.
    pub fn recommend_team(&self) -> Option<&TeamPerformance> {
        self.teams
            .values()
            .filter(|t| t.sample_count >= 2)
            .max_by(|a, b| {
                a.composite_score()
                    .partial_cmp(&b.composite_score())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Recommend the best agent pair.
    pub fn recommend_pair(&self) -> Option<&PairPerformance> {
        self.pairs
            .values()
            .filter(|p| p.sample_count >= 2)
            .max_by(|a, b| {
                a.ema_synergy
                    .partial_cmp(&b.ema_synergy)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Recommend the best workflow.
    pub fn recommend_workflow(&self) -> Option<&WorkflowPerformance> {
        self.workflows
            .values()
            .filter(|w| w.sample_count >= 2)
            .max_by(|a, b| {
                a.ema_success_rate
                    .partial_cmp(&b.ema_success_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Get team performance.
    pub fn get_team(&self, key: &str) -> Option<&TeamPerformance> {
        self.teams.get(key)
    }

    /// Get pair performance.
    pub fn get_pair(&self, key: &str) -> Option<&PairPerformance> {
        self.pairs.get(key)
    }

    /// Get workflow performance.
    pub fn get_workflow(&self, key: &str) -> Option<&WorkflowPerformance> {
        self.workflows.get(key)
    }

    /// Get total tracked entities.
    pub fn total_tracked(&self) -> usize {
        self.teams.len() + self.pairs.len() + self.workflows.len()
    }
}

impl Default for OrgLearningEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn ema(current: f64, new_value: f64) -> f64 {
    EMA_ALPHA * new_value + (1.0 - EMA_ALPHA) * current
}
