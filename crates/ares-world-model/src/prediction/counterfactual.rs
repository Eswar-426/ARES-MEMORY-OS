use super::models::{
    Counterfactual, CounterfactualResult, CounterfactualSummary, CounterfactualType,
};
use crate::simulation::models::SimulationResult;

/// Counterfactual engine — "What if?" analysis.
///
/// Applies perturbations to a simulation result and recalculates
/// outcomes to understand resilience and vulnerability.
/// Fully deterministic — no LLM dependency.
pub struct CounterfactualEngine;

impl Default for CounterfactualEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CounterfactualEngine {
    pub fn new() -> Self {
        Self
    }

    /// Evaluate a single counterfactual against a simulation result.
    pub fn evaluate(
        &self,
        counterfactual: &Counterfactual,
        simulation: &SimulationResult,
    ) -> CounterfactualResult {
        let (adj_success, adj_cost, adj_duration) =
            self.apply_perturbation(counterfactual, simulation);

        let impact_score = self.calculate_impact(
            simulation.success_probability,
            adj_success,
            simulation.total_cost,
            adj_cost,
        );

        let mitigation_suggestions = self.suggest_mitigations(counterfactual, impact_score);

        CounterfactualResult {
            counterfactual: counterfactual.clone(),
            original_success_probability: simulation.success_probability,
            adjusted_success_probability: adj_success,
            original_cost: simulation.total_cost,
            adjusted_cost: adj_cost,
            original_duration_secs: simulation.task_duration_secs,
            adjusted_duration_secs: adj_duration,
            impact_score,
            mitigation_suggestions,
        }
    }

    /// Evaluate all standard counterfactuals against a simulation.
    pub fn evaluate_standard(&self, simulation: &SimulationResult) -> Vec<CounterfactualResult> {
        CounterfactualType::standard_counterfactuals()
            .into_iter()
            .enumerate()
            .map(|(i, ct)| {
                let counterfactual = self.create_standard_counterfactual(i, ct);
                self.evaluate(&counterfactual, simulation)
            })
            .collect()
    }

    /// Summarize a batch of counterfactual results.
    pub fn summarize(&self, results: &[CounterfactualResult]) -> CounterfactualSummary {
        let significant_count = results.iter().filter(|r| r.is_significant()).count();
        let critical_count = results.iter().filter(|r| r.is_critical()).count();

        let most_impactful = results
            .iter()
            .max_by(|a, b| {
                a.impact_score
                    .partial_cmp(&b.impact_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|r| r.counterfactual.description.clone());

        let average_impact = if results.is_empty() {
            0.0
        } else {
            results.iter().map(|r| r.impact_score).sum::<f64>() / results.len() as f64
        };

        CounterfactualSummary {
            total_evaluated: results.len(),
            significant_count,
            critical_count,
            most_impactful,
            average_impact,
        }
    }

    /// Apply the perturbation to simulation values.
    fn apply_perturbation(
        &self,
        counterfactual: &Counterfactual,
        simulation: &SimulationResult,
    ) -> (f64, f64, f64) {
        let param = counterfactual.parameter.clamp(0.0, 1.0);

        match counterfactual.counterfactual_type {
            CounterfactualType::AgentFailure => {
                // Agent fails: success drops, duration increases, cost stays
                let success = simulation.success_probability * (1.0 - param * 0.4);
                let duration = simulation.task_duration_secs * (1.0 + param * 0.5);
                (success.clamp(0.01, 0.99), simulation.total_cost, duration)
            }
            CounterfactualType::ProviderUnavailable => {
                // Provider down: success drops significantly, cost increases
                let success = simulation.success_probability * (1.0 - param * 0.5);
                let cost = simulation.total_cost * (1.0 + param * 0.3);
                let duration = simulation.task_duration_secs * (1.0 + param * 0.3);
                (success.clamp(0.01, 0.99), cost, duration)
            }
            CounterfactualType::BudgetReduction => {
                // Budget reduced: success drops, cost constrained
                let cost = simulation.total_cost * (1.0 - param * 0.5);
                let success = simulation.success_probability * (1.0 - param * 0.3);
                let duration = simulation.task_duration_secs * (1.0 + param * 0.2);
                (success.clamp(0.01, 0.99), cost.max(0.0), duration)
            }
            CounterfactualType::ToolAccessLost => {
                // Tools unavailable: success drops, duration increases significantly
                let success = simulation.success_probability * (1.0 - param * 0.35);
                let duration = simulation.task_duration_secs * (1.0 + param * 0.8);
                let cost = simulation.total_cost * (1.0 + param * 0.2);
                (success.clamp(0.01, 0.99), cost, duration)
            }
            CounterfactualType::DeadlineTightened => {
                // Tighter deadline: success drops, cost increases (rush), duration forced down
                let success = simulation.success_probability * (1.0 - param * 0.25);
                let cost = simulation.total_cost * (1.0 + param * 0.4);
                let duration = simulation.task_duration_secs * (1.0 - param * 0.3);
                (success.clamp(0.01, 0.99), cost, duration.max(60.0))
            }
            CounterfactualType::ResourceReduction => {
                let success = simulation.success_probability * (1.0 - param * 0.3);
                let duration = simulation.task_duration_secs * (1.0 + param * 0.4);
                (success.clamp(0.01, 0.99), simulation.total_cost, duration)
            }
            CounterfactualType::QualityIncrease => {
                // Higher quality demand: cost and duration increase, success may drop slightly
                let cost = simulation.total_cost * (1.0 + param * 0.5);
                let duration = simulation.task_duration_secs * (1.0 + param * 0.4);
                let success = simulation.success_probability * (1.0 - param * 0.1);
                (success.clamp(0.01, 0.99), cost, duration)
            }
            CounterfactualType::Custom(_) => {
                // Generic perturbation
                let success = simulation.success_probability * (1.0 - param * 0.2);
                (
                    success.clamp(0.01, 0.99),
                    simulation.total_cost,
                    simulation.task_duration_secs,
                )
            }
        }
    }

    /// Calculate the normalized impact of a perturbation.
    fn calculate_impact(
        &self,
        orig_success: f64,
        adj_success: f64,
        orig_cost: f64,
        adj_cost: f64,
    ) -> f64 {
        let success_impact = (orig_success - adj_success).abs();
        let cost_impact = if orig_cost > 0.0 {
            ((adj_cost - orig_cost) / orig_cost).abs().min(1.0)
        } else {
            0.0
        };

        // Success impact weighted more heavily
        (success_impact * 0.7 + cost_impact * 0.3).clamp(0.0, 1.0)
    }

    /// Create a standard counterfactual scenario from its type.
    fn create_standard_counterfactual(&self, idx: usize, ct: CounterfactualType) -> Counterfactual {
        let (description, parameter) = match &ct {
            CounterfactualType::AgentFailure => ("What if a primary agent fails?".to_string(), 0.5),
            CounterfactualType::ProviderUnavailable => (
                "What if the AI provider becomes unavailable?".to_string(),
                0.7,
            ),
            CounterfactualType::BudgetReduction => {
                ("What if budget drops by 50%?".to_string(), 0.5)
            }
            CounterfactualType::ToolAccessLost => ("What if tool access is lost?".to_string(), 0.6),
            CounterfactualType::DeadlineTightened => {
                ("What if the deadline is halved?".to_string(), 0.5)
            }
            _ => ("What if conditions change?".to_string(), 0.5),
        };

        Counterfactual {
            id: format!("cf_{}", idx),
            counterfactual_type: ct,
            description,
            parameter,
        }
    }

    /// Suggest mitigations based on counterfactual type and impact.
    fn suggest_mitigations(
        &self,
        counterfactual: &Counterfactual,
        impact_score: f64,
    ) -> Vec<String> {
        if impact_score < 0.05 {
            return vec!["No mitigation needed — minimal impact".to_string()];
        }

        match counterfactual.counterfactual_type {
            CounterfactualType::AgentFailure => vec![
                "Configure agent failover with backup agents".to_string(),
                "Add health checks and auto-recovery".to_string(),
            ],
            CounterfactualType::ProviderUnavailable => vec![
                "Enable multi-provider fallback".to_string(),
                "Cache critical responses for offline operation".to_string(),
                "Use local deterministic fallbacks".to_string(),
            ],
            CounterfactualType::BudgetReduction => vec![
                "Identify and prioritize essential steps".to_string(),
                "Switch to cheapest scenario variant".to_string(),
            ],
            CounterfactualType::ToolAccessLost => vec![
                "Implement manual fallback procedures".to_string(),
                "Cache tool outputs where possible".to_string(),
            ],
            CounterfactualType::DeadlineTightened => vec![
                "Identify parallelizable steps".to_string(),
                "Cut non-essential quality checks".to_string(),
                "Use fastest scenario variant".to_string(),
            ],
            _ => vec!["Review scenario constraints and adjust".to_string()],
        }
    }
}
