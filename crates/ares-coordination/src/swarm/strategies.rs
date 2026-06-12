use serde::{Deserialize, Serialize};

/// Strategy for swarm coordination.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwarmStrategy {
    /// All agents work simultaneously, best result wins.
    Parallel,
    /// Leaders delegate, workers execute, results merge upward.
    Hierarchical,
    /// Dynamically reassign agents based on intermediate results.
    Adaptive,
}

impl SwarmStrategy {
    pub fn description(&self) -> &str {
        match self {
            SwarmStrategy::Parallel => {
                "All agents work simultaneously on the same task; best result wins"
            }
            SwarmStrategy::Hierarchical => {
                "Leaders delegate sub-tasks; workers execute; results merge upward"
            }
            SwarmStrategy::Adaptive => "Dynamically reassign agents based on intermediate results",
        }
    }

    pub fn recommended_min_agents(&self) -> usize {
        match self {
            SwarmStrategy::Parallel => 2,
            SwarmStrategy::Hierarchical => 3,
            SwarmStrategy::Adaptive => 3,
        }
    }
}
