use super::models::{WorldState, WorldStateDiff};

/// Engine for capturing, diffing, and managing world state snapshots.
pub struct WorldStateEngine;

impl Default for WorldStateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldStateEngine {
    pub fn new() -> Self {
        Self
    }

    /// Capture the current world state (from provided data).
    pub fn capture_state(&self, state: WorldState) -> WorldState {
        state
    }

    /// Compute the difference between two world states.
    pub fn diff_states(&self, old: &WorldState, new: &WorldState) -> WorldStateDiff {
        let old_goal_ids: std::collections::HashSet<&str> =
            old.goals.iter().map(|g| g.id.as_str()).collect();
        let new_goal_ids: std::collections::HashSet<&str> =
            new.goals.iter().map(|g| g.id.as_str()).collect();

        let goals_added = new_goal_ids.difference(&old_goal_ids).count();
        let goals_removed = old_goal_ids.difference(&new_goal_ids).count();

        let old_resource_names: std::collections::HashSet<&str> =
            old.resources.iter().map(|r| r.name.as_str()).collect();
        let new_resource_names: std::collections::HashSet<&str> =
            new.resources.iter().map(|r| r.name.as_str()).collect();
        let resources_changed = old_resource_names
            .symmetric_difference(&new_resource_names)
            .count()
            + old
                .resources
                .iter()
                .filter(|r| {
                    new.resources.iter().any(|nr| {
                        nr.name == r.name && (nr.available - r.available).abs() > f64::EPSILON
                    })
                })
                .count();

        let old_agent_ids: std::collections::HashSet<&str> =
            old.active_agents.iter().map(|a| a.id.as_str()).collect();
        let new_agent_ids: std::collections::HashSet<&str> =
            new.active_agents.iter().map(|a| a.id.as_str()).collect();
        let agents_changed = old_agent_ids.symmetric_difference(&new_agent_ids).count();

        let old_constraint_names: std::collections::HashSet<&str> =
            old.constraints.iter().map(|c| c.name.as_str()).collect();
        let new_constraint_names: std::collections::HashSet<&str> =
            new.constraints.iter().map(|c| c.name.as_str()).collect();
        let constraints_changed = old_constraint_names
            .symmetric_difference(&new_constraint_names)
            .count();

        let budget_delta = new.total_budget() - old.total_budget();
        let agent_count_delta = new.active_agents.len() as i64 - old.active_agents.len() as i64;

        WorldStateDiff {
            goals_added,
            goals_removed,
            resources_changed,
            agents_changed,
            constraints_changed,
            budget_delta,
            agent_count_delta,
        }
    }

    /// Serialize a world state to JSON.
    pub fn to_json(&self, state: &WorldState) -> Result<String, serde_json::Error> {
        serde_json::to_string(state)
    }

    /// Deserialize a world state from JSON.
    pub fn from_json(&self, json: &str) -> Result<WorldState, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Validate that a world state is internally consistent.
    pub fn validate(&self, state: &WorldState) -> Vec<String> {
        let mut issues = Vec::new();

        for resource in &state.resources {
            if resource.available < 0.0 {
                issues.push(format!(
                    "Resource '{}' has negative availability",
                    resource.name
                ));
            }
            if resource.capacity < 0.0 {
                issues.push(format!(
                    "Resource '{}' has negative capacity",
                    resource.name
                ));
            }
            if resource.available > resource.capacity {
                issues.push(format!(
                    "Resource '{}' availability exceeds capacity",
                    resource.name
                ));
            }
        }

        for agent in &state.active_agents {
            if agent.success_rate < 0.0 || agent.success_rate > 1.0 {
                issues.push(format!(
                    "Agent '{}' has invalid success rate: {}",
                    agent.name, agent.success_rate
                ));
            }
        }

        for constraint in &state.constraints {
            if constraint.is_hard && constraint.is_violated() {
                issues.push(format!("Hard constraint '{}' is violated", constraint.name));
            }
        }

        issues
    }
}
