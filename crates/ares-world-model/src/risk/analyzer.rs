use super::models::{RiskCategory, RiskFactor, RiskLevel, RiskReport};
use crate::scenario::models::Scenario;
use crate::simulation::models::SimulationResult;
use crate::state::models::WorldState;
use ares_core::RiskReportId;
use chrono::Utc;

/// Multi-dimensional risk analyzer. Deterministic — no LLM dependency.
///
/// Analyzes: failure risk, budget risk, resource risk, dependency risk, execution risk.
pub struct RiskAnalyzer;

impl Default for RiskAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl RiskAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Perform a comprehensive risk analysis on a scenario + simulation result.
    pub fn analyze(
        &self,
        scenario: &Scenario,
        simulation: &SimulationResult,
        world_state: &WorldState,
    ) -> RiskReport {
        let mut risk_factors = Vec::new();

        let failure_probability = self.assess_failure_risk(scenario, simulation, &mut risk_factors);
        let budget_overrun_probability =
            self.assess_budget_risk(scenario, world_state, &mut risk_factors);
        let resource_exhaustion_risk =
            self.assess_resource_risk(scenario, world_state, &mut risk_factors);
        let dependency_risk = self.assess_dependency_risk(scenario, &mut risk_factors);
        let execution_risk = self.assess_execution_risk(scenario, simulation, &mut risk_factors);

        let overall_score = 0.3 * failure_probability
            + 0.2 * budget_overrun_probability
            + 0.2 * resource_exhaustion_risk
            + 0.15 * dependency_risk
            + 0.15 * execution_risk;

        let overall_risk = RiskLevel::from_score(overall_score);
        let mitigations = self.suggest_mitigations(&risk_factors, world_state);

        RiskReport {
            id: RiskReportId::new(),
            scenario_id: scenario.id.clone(),
            overall_risk,
            failure_probability,
            budget_overrun_probability,
            resource_exhaustion_risk,
            dependency_risk,
            execution_risk,
            risk_factors,
            mitigations,
            analyzed_at: Utc::now(),
        }
    }

    /// Assess failure risk based on success probability and step count.
    fn assess_failure_risk(
        &self,
        scenario: &Scenario,
        simulation: &SimulationResult,
        factors: &mut Vec<RiskFactor>,
    ) -> f64 {
        let base_failure = 1.0 - simulation.success_probability;

        // More steps = more potential failure points
        let step_penalty = (scenario.steps.len() as f64 * 0.02).min(0.2);

        if base_failure > 0.3 {
            factors.push(RiskFactor {
                category: RiskCategory::Failure,
                description: format!(
                    "Low success probability: {:.0}%",
                    simulation.success_probability * 100.0
                ),
                severity: base_failure,
                likelihood: 0.8,
            });
        }

        if scenario.steps.len() > 10 {
            factors.push(RiskFactor {
                category: RiskCategory::Failure,
                description: format!(
                    "High step count ({}) increases failure surface",
                    scenario.steps.len()
                ),
                severity: step_penalty,
                likelihood: 0.6,
            });
        }

        (base_failure + step_penalty).clamp(0.0, 1.0)
    }

    /// Assess budget overrun risk.
    fn assess_budget_risk(
        &self,
        scenario: &Scenario,
        world_state: &WorldState,
        factors: &mut Vec<RiskFactor>,
    ) -> f64 {
        let budget = world_state.total_budget();
        if budget <= 0.0 {
            return 0.1; // No budget = low budget risk (but not zero)
        }

        let cost_ratio = scenario.estimated_cost / budget;

        if cost_ratio > 1.0 {
            factors.push(RiskFactor {
                category: RiskCategory::Budget,
                description: format!(
                    "Estimated cost ${:.2} exceeds budget ${:.2} ({:.0}% over)",
                    scenario.estimated_cost,
                    budget,
                    (cost_ratio - 1.0) * 100.0
                ),
                severity: 0.9,
                likelihood: 0.9,
            });
            return 0.9;
        }

        if cost_ratio > 0.8 {
            factors.push(RiskFactor {
                category: RiskCategory::Budget,
                description: format!(
                    "Cost uses {:.0}% of budget (tight margin)",
                    cost_ratio * 100.0
                ),
                severity: 0.5,
                likelihood: 0.6,
            });
            return cost_ratio * 0.5;
        }

        cost_ratio * 0.2
    }

    /// Assess resource exhaustion risk.
    fn assess_resource_risk(
        &self,
        scenario: &Scenario,
        world_state: &WorldState,
        factors: &mut Vec<RiskFactor>,
    ) -> f64 {
        let mut max_risk = 0.0_f64;

        // Agent availability
        let needed = scenario.agent_assignments.len();
        let available = world_state.available_agent_count();
        if needed > available {
            let severity = ((needed - available) as f64 / needed.max(1) as f64).min(1.0);
            factors.push(RiskFactor {
                category: RiskCategory::Resource,
                description: format!("Need {} agents but only {} available", needed, available),
                severity,
                likelihood: 0.9,
            });
            max_risk = max_risk.max(severity);
        }

        // Resource utilization
        for resource in &world_state.resources {
            let util = resource.utilization();
            if util > 0.9 {
                factors.push(RiskFactor {
                    category: RiskCategory::Resource,
                    description: format!(
                        "Resource '{}' at {:.0}% utilization",
                        resource.name,
                        util * 100.0
                    ),
                    severity: util,
                    likelihood: 0.7,
                });
                max_risk = max_risk.max(util * 0.8);
            }
        }

        max_risk.clamp(0.0, 1.0)
    }

    /// Assess dependency risk based on step chains and agent dependencies.
    fn assess_dependency_risk(&self, scenario: &Scenario, factors: &mut Vec<RiskFactor>) -> f64 {
        let step_count = scenario.steps.len();
        if step_count <= 1 {
            return 0.0;
        }

        // Linear dependency chain risk: each step depends on the previous
        let chain_length = step_count as f64;
        let chain_risk = (chain_length * 0.03).min(0.5);

        // Single-agent dependency: all steps assigned to one agent
        let unique_agents: std::collections::HashSet<&str> = scenario
            .steps
            .iter()
            .filter_map(|s| s.agent.as_deref())
            .collect();

        let agent_concentration = if unique_agents.len() <= 1 && step_count > 3 {
            factors.push(RiskFactor {
                category: RiskCategory::Dependency,
                description: "All steps depend on a single agent".to_string(),
                severity: 0.6,
                likelihood: 0.5,
            });
            0.3
        } else {
            0.0
        };

        (chain_risk + agent_concentration).clamp(0.0, 1.0)
    }

    /// Assess execution risk from simulation properties.
    fn assess_execution_risk(
        &self,
        scenario: &Scenario,
        simulation: &SimulationResult,
        factors: &mut Vec<RiskFactor>,
    ) -> f64 {
        let mut risk = 0.0;

        // High risk score from simulation
        if simulation.risk_score > 0.5 {
            factors.push(RiskFactor {
                category: RiskCategory::Execution,
                description: format!(
                    "Simulation risk score is high: {:.2}",
                    simulation.risk_score
                ),
                severity: simulation.risk_score,
                likelihood: 0.7,
            });
            risk += simulation.risk_score * 0.4;
        }

        // Long duration risk
        if simulation.task_duration_secs > 14400.0 {
            // > 4 hours
            factors.push(RiskFactor {
                category: RiskCategory::Execution,
                description: format!(
                    "Long execution time: {:.1} hours",
                    simulation.task_duration_secs / 3600.0
                ),
                severity: 0.5,
                likelihood: 0.5,
            });
            risk += 0.15;
        }

        // Low quality scenario
        if scenario.estimated_quality < 0.5 {
            factors.push(RiskFactor {
                category: RiskCategory::Execution,
                description: format!(
                    "Low quality estimate: {:.0}%",
                    scenario.estimated_quality * 100.0
                ),
                severity: 1.0 - scenario.estimated_quality,
                likelihood: 0.6,
            });
            risk += 0.1;
        }

        risk.clamp(0.0, 1.0)
    }

    fn suggest_mitigations(&self, factors: &[RiskFactor], world_state: &WorldState) -> Vec<String> {
        let mut mitigations = Vec::new();

        for factor in factors {
            match factor.category {
                RiskCategory::Budget => {
                    mitigations.push("Consider the 'cheapest' scenario variant".to_string());
                    if world_state.total_budget() > 0.0 {
                        mitigations.push("Increase budget or reduce scope".to_string());
                    }
                }
                RiskCategory::Resource => {
                    mitigations.push("Scale up available agents or resources".to_string());
                }
                RiskCategory::Failure => {
                    mitigations.push("Add checkpoints and rollback capability".to_string());
                    mitigations.push("Use 'reliability_first' strategy".to_string());
                }
                RiskCategory::Dependency => {
                    mitigations.push("Distribute steps across multiple agents".to_string());
                }
                RiskCategory::Execution => {
                    mitigations
                        .push("Break into smaller phases with intermediate validation".to_string());
                }
            }
        }

        mitigations.sort();
        mitigations.dedup();
        mitigations
    }
}
