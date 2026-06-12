use super::models::{Scenario, ScenarioGenerationConfig, ScenarioStep, ScenarioType};
use crate::state::models::WorldState;
use ares_core::ScenarioId;
use chrono::Utc;

/// Generates multiple possible futures (scenarios) from a goal and world state.
///
/// All generation is deterministic — no LLM dependency. Uses heuristics
/// based on goal properties, resource availability, and agent capacity.
pub struct ScenarioGenerator;

impl Default for ScenarioGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ScenarioGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate a set of scenarios for the given goal within the world state.
    pub fn generate(
        &self,
        goal_id: &str,
        goal_title: &str,
        world_state: &WorldState,
        config: &ScenarioGenerationConfig,
    ) -> Vec<Scenario> {
        let mut scenarios = Vec::new();

        if config.generate_all_standard {
            for scenario_type in ScenarioType::standard_types() {
                let scenario = self.generate_scenario(
                    goal_id,
                    goal_title,
                    world_state,
                    &scenario_type,
                    config,
                );
                scenarios.push(scenario);
            }
        }

        for custom_name in &config.custom_types {
            let scenario_type = ScenarioType::Custom(custom_name.clone());
            let scenario =
                self.generate_scenario(goal_id, goal_title, world_state, &scenario_type, config);
            scenarios.push(scenario);
        }

        scenarios
    }

    /// Generate a single scenario of the given type.
    fn generate_scenario(
        &self,
        goal_id: &str,
        goal_title: &str,
        world_state: &WorldState,
        scenario_type: &ScenarioType,
        config: &ScenarioGenerationConfig,
    ) -> Scenario {
        let base_steps = self.estimate_base_steps(goal_title);
        let (cost_factor, duration_factor, quality_factor) = self.type_factors(scenario_type);

        let agent_count = world_state.available_agent_count().max(1);
        let budget = world_state.total_budget();

        let steps = self.generate_steps(
            goal_title,
            base_steps,
            cost_factor * config.cost_multiplier,
            duration_factor * config.duration_multiplier,
            agent_count,
            config.max_steps,
        );

        let estimated_cost: f64 = steps.iter().map(|s| s.cost).sum();
        let estimated_duration_secs: f64 = match scenario_type {
            ScenarioType::Fastest => {
                // Parallel execution: use max step duration instead of sum
                steps
                    .iter()
                    .map(|s| s.duration_secs)
                    .fold(0.0_f64, f64::max)
                    * (steps.len() as f64 / agent_count.max(1) as f64).ceil()
            }
            _ => steps.iter().map(|s| s.duration_secs).sum(),
        };

        let estimated_quality = (quality_factor * 0.8).min(1.0);

        let agent_assignments: Vec<String> = world_state
            .active_agents
            .iter()
            .filter(|a| a.status == "ready" || a.status == "idle")
            .take(agent_count)
            .map(|a| a.id.clone())
            .collect();

        let description = format!(
            "{} strategy for '{}': {} steps, est. ${:.2}, {:.0}s, {:.0}% quality{}",
            scenario_type,
            goal_title,
            steps.len(),
            estimated_cost,
            estimated_duration_secs,
            estimated_quality * 100.0,
            if budget > 0.0 && estimated_cost > budget {
                " (exceeds budget!)"
            } else {
                ""
            },
        );

        Scenario {
            id: ScenarioId::new(),
            goal_id: goal_id.to_string(),
            scenario_type: scenario_type.clone(),
            description,
            estimated_cost,
            estimated_duration_secs,
            estimated_quality,
            agent_assignments,
            steps,
            created_at: Utc::now(),
        }
    }

    /// Estimate the base number of steps from the goal title length and keyword count.
    fn estimate_base_steps(&self, goal_title: &str) -> u32 {
        let word_count = goal_title.split_whitespace().count();
        let complexity_keywords = [
            "full",
            "complete",
            "entire",
            "comprehensive",
            "production",
            "multi",
            "distributed",
            "scalable",
            "enterprise",
        ];
        let keyword_bonus: u32 = complexity_keywords
            .iter()
            .filter(|kw| goal_title.to_lowercase().contains(**kw))
            .count() as u32;

        let base = match word_count {
            0..=2 => 3,
            3..=5 => 5,
            6..=10 => 7,
            _ => 10,
        };

        base + keyword_bonus
    }

    /// Return (cost_factor, duration_factor, quality_factor) for each scenario type.
    fn type_factors(&self, scenario_type: &ScenarioType) -> (f64, f64, f64) {
        match scenario_type {
            ScenarioType::Fastest => (1.5, 0.5, 0.7), // expensive, fast, lower quality
            ScenarioType::Cheapest => (0.4, 1.8, 0.6), // cheap, slow, lower quality
            ScenarioType::HighestQuality => (1.3, 1.5, 1.0), // expensive, slow, high quality
            ScenarioType::Balanced => (1.0, 1.0, 0.85), // balanced
            ScenarioType::Custom(_) => (1.0, 1.0, 0.8), // default
        }
    }

    /// Generate concrete steps for a scenario.
    fn generate_steps(
        &self,
        goal_title: &str,
        base_count: u32,
        cost_factor: f64,
        duration_factor: f64,
        agent_count: usize,
        max_steps: u32,
    ) -> Vec<ScenarioStep> {
        let step_count = base_count.min(max_steps);
        let base_cost_per_step = 2.0 * cost_factor;
        let base_duration_per_step = 300.0 * duration_factor; // 5 minutes base

        let step_templates = [
            "Analyze requirements",
            "Set up environment",
            "Design architecture",
            "Implement core logic",
            "Build components",
            "Write tests",
            "Integration testing",
            "Code review",
            "Optimization pass",
            "Documentation",
            "Deployment preparation",
            "Final validation",
            "Performance testing",
            "Security review",
            "Release",
        ];

        (0..step_count)
            .map(|i| {
                let template_idx = i as usize % step_templates.len();
                let title = format!(
                    "{} for '{}'",
                    step_templates[template_idx],
                    if goal_title.len() > 30 {
                        &goal_title[..30]
                    } else {
                        goal_title
                    }
                );

                // Later steps tend to be slightly more expensive and longer
                let step_weight = 1.0 + (i as f64 * 0.1);
                let agent_idx = i as usize % agent_count;

                ScenarioStep {
                    order: i + 1,
                    title,
                    duration_secs: base_duration_per_step * step_weight,
                    cost: base_cost_per_step * step_weight,
                    agent: Some(format!("agent_{}", agent_idx)),
                }
            })
            .collect()
    }
}
