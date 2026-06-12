use ares_agent_runtime::models::{AgentId, AgentRole};
use std::collections::HashMap;

use super::models::{AgentReputation, TaskOutcome};

/// EMA alpha for reputation updates.
const EMA_ALPHA: f64 = 0.2;

/// Tracks and updates agent reputations using EMA.
pub struct ReputationTracker {
    reputations: HashMap<AgentId, AgentReputation>,
}

impl ReputationTracker {
    pub fn new() -> Self {
        Self {
            reputations: HashMap::new(),
        }
    }

    /// Record a task outcome and update the agent's reputation via EMA.
    pub fn record_outcome(&mut self, agent_id: AgentId, outcome: &TaskOutcome) {
        let rep = self.reputations.entry(agent_id).or_default();

        let success_val = if outcome.success { 1.0 } else { 0.0 };

        if rep.task_count == 0 {
            rep.success_rate = success_val;
            rep.avg_latency_ms = outcome.latency_ms;
            rep.cost_efficiency = if outcome.cost > 0.0 {
                (outcome.quality / outcome.cost).min(1.0)
            } else {
                1.0
            };
            rep.quality_score = outcome.quality;
        } else {
            rep.success_rate = ema(rep.success_rate, success_val);
            rep.avg_latency_ms = ema(rep.avg_latency_ms, outcome.latency_ms);
            rep.cost_efficiency = ema(
                rep.cost_efficiency,
                if outcome.cost > 0.0 {
                    (outcome.quality / outcome.cost).min(1.0)
                } else {
                    1.0
                },
            );
            rep.quality_score = ema(rep.quality_score, outcome.quality);
        }

        // Reliability: availability heuristic (successful outcomes increase it)
        rep.reliability = ema(rep.reliability, if outcome.success { 1.0 } else { 0.7 });
        rep.task_count += 1;
        rep.updated_at = chrono::Utc::now().timestamp();
    }

    /// Get an agent's reputation.
    pub fn get_reputation(&self, agent_id: &AgentId) -> Option<&AgentReputation> {
        self.reputations.get(agent_id)
    }

    /// Rank all agents with a given role by composite score.
    pub fn rank_agents(&self, _role: &AgentRole) -> Vec<(AgentId, f64)> {
        let mut ranked: Vec<(AgentId, f64)> = self
            .reputations
            .iter()
            .map(|(id, rep)| (*id, rep.composite_score()))
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked
    }

    /// Apply time-based decay to an agent's reputation.
    pub fn decay_reputation(&mut self, agent_id: &AgentId, decay_factor: f64) {
        if let Some(rep) = self.reputations.get_mut(agent_id) {
            rep.success_rate *= decay_factor;
            rep.quality_score *= decay_factor;
            rep.reliability *= decay_factor;
            rep.cost_efficiency *= decay_factor;
            rep.updated_at = chrono::Utc::now().timestamp();
        }
    }

    /// Get the best agent by composite score.
    pub fn best_agent(&self) -> Option<(AgentId, f64)> {
        self.reputations
            .iter()
            .map(|(id, rep)| (*id, rep.composite_score()))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Get total tracked agents.
    pub fn agent_count(&self) -> usize {
        self.reputations.len()
    }

    /// Check if an agent has been tracked.
    pub fn has_agent(&self, agent_id: &AgentId) -> bool {
        self.reputations.contains_key(agent_id)
    }
}

impl Default for ReputationTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Exponential Moving Average update.
fn ema(current: f64, new_value: f64) -> f64 {
    EMA_ALPHA * new_value + (1.0 - EMA_ALPHA) * current
}
