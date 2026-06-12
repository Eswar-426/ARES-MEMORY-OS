use ares_agent_runtime::models::AgentId;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::rules::{GovernorDecision, GovernorRules};

/// Rate tracking for a single agent.
struct AgentRateTracker {
    message_count: u32,
    window_start: i64,
}

/// Safety governor that enforces coordination rules before every
/// delegation, swarm launch, consensus round, and debate round.
pub struct SafetyGovernor {
    rules: GovernorRules,
    agent_rates: Arc<RwLock<HashMap<AgentId, AgentRateTracker>>>,
    active_tasks: AtomicU32,
    total_cost: Arc<RwLock<f64>>,
    decisions_log: Arc<RwLock<Vec<GovernorLog>>>,
}

/// Log entry for governor decisions.
#[derive(Debug, Clone)]
pub struct GovernorLog {
    pub check_type: String,
    pub decision: GovernorDecision,
    pub timestamp: i64,
}

impl SafetyGovernor {
    pub fn new(rules: GovernorRules) -> Self {
        Self {
            rules,
            agent_rates: Arc::new(RwLock::new(HashMap::new())),
            active_tasks: AtomicU32::new(0),
            total_cost: Arc::new(RwLock::new(0.0)),
            decisions_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Check if a delegation at the given depth is allowed.
    pub async fn check_delegation(&self, depth: u32) -> GovernorDecision {
        let decision = if depth > self.rules.max_delegation_depth {
            GovernorDecision::Deny(format!(
                "Delegation depth {} exceeds maximum {}",
                depth, self.rules.max_delegation_depth
            ))
        } else if depth == self.rules.max_delegation_depth {
            GovernorDecision::Throttle(500) // Near limit, slow down
        } else {
            GovernorDecision::Allow
        };

        self.log("delegation", &decision).await;
        decision
    }

    /// Check if an agent's message rate is within limits.
    pub async fn check_message_rate(&self, agent_id: &AgentId) -> GovernorDecision {
        let now = chrono::Utc::now().timestamp();
        let mut rates = self.agent_rates.write().await;

        let tracker = rates.entry(*agent_id).or_insert(AgentRateTracker {
            message_count: 0,
            window_start: now,
        });

        // Reset window if more than 60 seconds have passed
        if now - tracker.window_start >= 60 {
            tracker.message_count = 0;
            tracker.window_start = now;
        }

        tracker.message_count += 1;

        let decision = if tracker.message_count > self.rules.max_messages_per_minute {
            GovernorDecision::Deny(format!(
                "Agent {:?} exceeded message rate limit ({}/min)",
                agent_id, self.rules.max_messages_per_minute
            ))
        } else if tracker.message_count > self.rules.max_messages_per_minute * 80 / 100 {
            GovernorDecision::Throttle(100) // Approaching limit
        } else {
            GovernorDecision::Allow
        };

        self.log("message_rate", &decision).await;
        decision
    }

    /// Check if a swarm of the given size can be launched.
    pub async fn check_swarm_launch(&self, size: u32) -> GovernorDecision {
        let decision = if size > self.rules.max_swarm_size {
            GovernorDecision::Deny(format!(
                "Swarm size {} exceeds maximum {}",
                size, self.rules.max_swarm_size
            ))
        } else if size == self.rules.max_swarm_size {
            GovernorDecision::Throttle(200)
        } else {
            GovernorDecision::Allow
        };

        self.log("swarm_launch", &decision).await;
        decision
    }

    /// Check if a debate can continue for another round.
    pub async fn check_debate(&self, round_count: u32) -> GovernorDecision {
        let decision = if round_count >= self.rules.max_debate_rounds {
            GovernorDecision::Deny(format!(
                "Debate round {} exceeds maximum {}",
                round_count, self.rules.max_debate_rounds
            ))
        } else {
            GovernorDecision::Allow
        };

        self.log("debate", &decision).await;
        decision
    }

    /// Check if a consensus can continue for another round.
    pub async fn check_consensus(&self, round_count: u32) -> GovernorDecision {
        let decision = if round_count >= self.rules.max_consensus_rounds {
            GovernorDecision::Deny(format!(
                "Consensus round {} exceeds maximum {}",
                round_count, self.rules.max_consensus_rounds
            ))
        } else {
            GovernorDecision::Allow
        };

        self.log("consensus", &decision).await;
        decision
    }

    /// Check if the projected cost is within budget.
    pub async fn check_cost(&self, projected_additional: f64) -> GovernorDecision {
        let current = *self.total_cost.read().await;
        let total = current + projected_additional;

        let decision = if total > self.rules.max_execution_cost {
            GovernorDecision::Deny(format!(
                "Total cost {:.2} would exceed budget {:.2}",
                total, self.rules.max_execution_cost
            ))
        } else if total > self.rules.max_execution_cost * 0.9 {
            GovernorDecision::Throttle(300)
        } else {
            GovernorDecision::Allow
        };

        self.log("cost", &decision).await;
        decision
    }

    /// Check if a new task can be started.
    pub async fn check_concurrent_tasks(&self) -> GovernorDecision {
        let current = self.active_tasks.load(Ordering::Relaxed);
        let decision = if current >= self.rules.max_concurrent_tasks {
            GovernorDecision::Deny(format!(
                "Active tasks {} at maximum {}",
                current, self.rules.max_concurrent_tasks
            ))
        } else {
            GovernorDecision::Allow
        };

        self.log("concurrent_tasks", &decision).await;
        decision
    }

    /// Check if the organization depth is within limits.
    pub async fn check_org_depth(&self, depth: u32) -> GovernorDecision {
        let decision = if depth > self.rules.max_org_depth {
            GovernorDecision::Deny(format!(
                "Organization depth {} exceeds maximum {}",
                depth, self.rules.max_org_depth
            ))
        } else {
            GovernorDecision::Allow
        };

        self.log("org_depth", &decision).await;
        decision
    }

    /// Record cost spent.
    pub async fn record_cost(&self, cost: f64) {
        let mut total = self.total_cost.write().await;
        *total += cost;
    }

    /// Increment active task count.
    pub fn task_started(&self) {
        self.active_tasks.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active task count.
    pub fn task_completed(&self) {
        self.active_tasks.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get the current active task count.
    pub fn active_task_count(&self) -> u32 {
        self.active_tasks.load(Ordering::Relaxed)
    }

    /// Get total cost recorded.
    pub async fn total_cost(&self) -> f64 {
        *self.total_cost.read().await
    }

    /// Get the decision log count.
    pub async fn decision_count(&self) -> usize {
        self.decisions_log.read().await.len()
    }

    /// Get the governor rules.
    pub fn rules(&self) -> &GovernorRules {
        &self.rules
    }

    async fn log(&self, check_type: &str, decision: &GovernorDecision) {
        self.decisions_log.write().await.push(GovernorLog {
            check_type: check_type.to_string(),
            decision: decision.clone(),
            timestamp: chrono::Utc::now().timestamp(),
        });
    }
}

impl Default for SafetyGovernor {
    fn default() -> Self {
        Self::new(GovernorRules::default())
    }
}
