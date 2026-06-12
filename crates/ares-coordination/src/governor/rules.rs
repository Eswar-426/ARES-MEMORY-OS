use serde::{Deserialize, Serialize};

/// Configurable governance rules for coordination safety.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernorRules {
    /// Maximum depth of delegation chains.
    pub max_delegation_depth: u32,
    /// Maximum messages per minute per agent.
    pub max_messages_per_minute: u32,
    /// Maximum number of agents in a swarm.
    pub max_swarm_size: u32,
    /// Maximum number of debate rounds.
    pub max_debate_rounds: u32,
    /// Maximum number of consensus rounds.
    pub max_consensus_rounds: u32,
    /// Maximum total execution cost per mission.
    pub max_execution_cost: f64,
    /// Maximum concurrent tasks system-wide.
    pub max_concurrent_tasks: u32,
    /// Maximum organization depth (hierarchy levels).
    pub max_org_depth: u32,
}

impl Default for GovernorRules {
    fn default() -> Self {
        Self {
            max_delegation_depth: 5,
            max_messages_per_minute: 100,
            max_swarm_size: 10,
            max_debate_rounds: 5,
            max_consensus_rounds: 3,
            max_execution_cost: 100.0,
            max_concurrent_tasks: 50,
            max_org_depth: 6,
        }
    }
}

/// Decision returned by the governor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GovernorDecision {
    /// Action is allowed.
    Allow,
    /// Action is denied with a reason.
    Deny(String),
    /// Action is allowed but should be throttled (delay in milliseconds).
    Throttle(u64),
}

impl GovernorDecision {
    pub fn is_allowed(&self) -> bool {
        matches!(
            self,
            GovernorDecision::Allow | GovernorDecision::Throttle(_)
        )
    }

    pub fn is_denied(&self) -> bool {
        matches!(self, GovernorDecision::Deny(_))
    }
}
