use super::models::{SimulationConfig, SimulationResult};
use crate::scenario::models::Scenario;
use crate::state::models::WorldState;
use ares_core::SimulationId;
use chrono::Utc;

/// Deterministic simulation engine. No LLM dependency.
///
/// Simulates a scenario against the current world state to estimate
/// cost, duration, success probability, resource usage, and risk.
pub struct SimulationEngine;

impl Default for SimulationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SimulationEngine {
    pub fn new() -> Self {
        Self
    }

    /// Simulate a single scenario against the world state.
    pub fn simulate(
        &self,
        scenario: &Scenario,
        world_state: &WorldState,
        config: &SimulationConfig,
    ) -> SimulationResult {
        let task_duration_secs = self.estimate_duration(scenario, world_state);
        let total_cost = self.estimate_cost(scenario, world_state);
        let agent_utilization = self.estimate_agent_utilization(scenario, world_state);
        let memory_usage_estimate = self.estimate_memory_usage(scenario);
        let risk_score = self.estimate_risk(scenario, world_state);
        let success_probability =
            self.estimate_success_probability(scenario, world_state, config, risk_score);

        SimulationResult {
            id: SimulationId::new(),
            scenario_id: scenario.id.clone(),
            task_duration_secs,
            total_cost,
            success_probability,
            agent_utilization,
            memory_usage_estimate,
            risk_score,
            simulated_at: Utc::now(),
        }
    }

    /// Simulate multiple scenarios and return results for all.
    pub fn simulate_batch(
        &self,
        scenarios: &[Scenario],
        world_state: &WorldState,
        config: &SimulationConfig,
    ) -> Vec<SimulationResult> {
        scenarios
            .iter()
            .map(|s| self.simulate(s, world_state, config))
            .collect()
    }

    /// Estimate total duration considering parallelism and agent availability.
    fn estimate_duration(&self, scenario: &Scenario, world_state: &WorldState) -> f64 {
        if scenario.steps.is_empty() {
            return 0.0;
        }

        let sequential_duration: f64 = scenario.steps.iter().map(|s| s.duration_secs).sum();
        let agent_count = world_state.available_agent_count().max(1) as f64;
        let step_count = scenario.steps.len() as f64;

        // Parallelism factor: more agents = shorter duration, but with coordination overhead
        let parallel_factor = (step_count / agent_count).ceil();
        let max_step_duration = scenario
            .steps
            .iter()
            .map(|s| s.duration_secs)
            .fold(0.0_f64, f64::max);

        let parallel_duration = max_step_duration * parallel_factor;

        // Use whichever is less: sequential or parallel estimation
        // Add 10% coordination overhead for parallel execution
        let effective_duration = if agent_count > 1.0 {
            (parallel_duration * 1.1).min(sequential_duration)
        } else {
            sequential_duration
        };

        effective_duration.max(0.0)
    }

    /// Estimate total cost from step costs and resource pricing.
    fn estimate_cost(&self, scenario: &Scenario, world_state: &WorldState) -> f64 {
        let base_cost: f64 = scenario.steps.iter().map(|s| s.cost).sum();

        // Add overhead for agent coordination
        let agent_count = scenario.agent_assignments.len().max(1) as f64;
        let coordination_overhead = if agent_count > 1.0 {
            base_cost * 0.05 * (agent_count - 1.0) // 5% overhead per additional agent
        } else {
            0.0
        };

        // Resource cost adjustment based on utilization
        let resource_factor = world_state
            .resources
            .iter()
            .map(|r| {
                let util = r.utilization();
                if util > 0.8 {
                    1.2 // high utilization = premium pricing
                } else {
                    1.0
                }
            })
            .fold(1.0_f64, f64::max);

        (base_cost + coordination_overhead) * resource_factor
    }

    /// Estimate success probability based on multiple factors.
    fn estimate_success_probability(
        &self,
        scenario: &Scenario,
        world_state: &WorldState,
        config: &SimulationConfig,
        risk_score: f64,
    ) -> f64 {
        // Base probability from scenario quality
        let quality_factor = scenario.estimated_quality;

        // Agent competence factor
        let agent_factor = world_state.average_agent_success_rate().max(0.5);

        // Complexity penalty: more steps = lower base probability
        let step_count = scenario.steps.len() as f64;
        let complexity_penalty = if step_count <= 3.0 {
            0.0
        } else if step_count <= 7.0 {
            0.05
        } else if step_count <= 12.0 {
            0.10
        } else {
            0.15
        };

        // Constraint satisfaction check
        let constraint_penalty = if world_state.has_violated_constraints() {
            0.2
        } else {
            0.0
        };

        // Budget feasibility
        let budget = world_state.total_budget();
        let budget_penalty = if budget > 0.0 && scenario.estimated_cost > budget {
            0.15
        } else {
            0.0
        };

        // Risk adjustment
        let risk_penalty = risk_score * 0.3;

        // Combine factors
        let mut probability = quality_factor * agent_factor
            - complexity_penalty
            - constraint_penalty
            - budget_penalty
            - risk_penalty;

        // Blend with historical data if available
        if let Some(hist_rate) = config.historical_success_rate {
            let weight = (config.historical_sample_count as f64 / 50.0).min(0.5);
            probability = probability * (1.0 - weight) + hist_rate * weight;
        }

        probability.clamp(0.01, 0.99)
    }

    /// Estimate agent utilization (0.0..=1.0).
    fn estimate_agent_utilization(&self, scenario: &Scenario, world_state: &WorldState) -> f64 {
        let available = world_state.available_agent_count().max(1) as f64;
        let assigned = scenario.agent_assignments.len() as f64;
        (assigned / available).min(1.0)
    }

    /// Estimate memory usage in MB (rough heuristic).
    fn estimate_memory_usage(&self, scenario: &Scenario) -> f64 {
        let base_mb = 50.0;
        let per_step_mb = 10.0;
        base_mb + (scenario.steps.len() as f64 * per_step_mb)
    }

    /// Estimate risk score (0.0..=1.0) based on scenario characteristics.
    fn estimate_risk(&self, scenario: &Scenario, world_state: &WorldState) -> f64 {
        let mut risk = 0.0;

        // Step complexity risk
        let step_count = scenario.steps.len() as f64;
        risk += (step_count * 0.03).min(0.3);

        // Budget risk
        let budget = world_state.total_budget();
        if budget > 0.0 {
            let cost_ratio = scenario.estimated_cost / budget;
            if cost_ratio > 1.0 {
                risk += 0.3;
            } else if cost_ratio > 0.8 {
                risk += 0.15;
            } else if cost_ratio > 0.5 {
                risk += 0.05;
            }
        }

        // Agent availability risk
        let available = world_state.available_agent_count();
        let needed = scenario.agent_assignments.len();
        if needed > available {
            risk += 0.2;
        }

        // Constraint risk
        if world_state.has_violated_constraints() {
            risk += 0.15;
        }

        risk.clamp(0.0, 1.0)
    }
}
