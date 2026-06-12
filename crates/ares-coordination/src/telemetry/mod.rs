use ares_agent_runtime::models::AgentId;

/// Telemetry for multi-agent coordination flows.
pub struct CoordinationMetrics;

impl CoordinationMetrics {
    pub fn new() -> Self {
        Self
    }

    pub fn record_message_sent(&self, from: &AgentId, to: Option<&AgentId>, msg_type: &str) {
        tracing::info!(
            from = ?from.0,
            to = ?to.map(|a| a.0),
            msg_type = msg_type,
            "coordination.message_sent"
        );
    }

    pub fn record_delegation(&self, task_id: &str, from: &AgentId, to: &AgentId) {
        tracing::info!(
            task_id = task_id,
            from = ?from.0,
            to = ?to.0,
            "coordination.delegation"
        );
    }

    pub fn record_consensus_decision(&self, topic: &str, algorithm: &str, agreed: bool) {
        tracing::info!(
            topic = topic,
            algorithm = algorithm,
            agreed = agreed,
            "coordination.consensus_decision"
        );
    }

    pub fn record_debate(&self, topic: &str, outcome: &str) {
        tracing::info!(topic = topic, outcome = outcome, "coordination.debate");
    }

    pub fn record_conflict(&self, conflict_type: &str, resolved: bool) {
        tracing::info!(
            conflict_type = conflict_type,
            resolved = resolved,
            "coordination.conflict"
        );
    }

    pub fn record_swarm_execution(&self, strategy: &str, agent_count: usize, success_rate: f64) {
        tracing::info!(
            strategy = strategy,
            agent_count = agent_count,
            success_rate = success_rate,
            "coordination.swarm_execution"
        );
    }

    pub fn record_verification(&self, pattern: &str, approved: bool) {
        tracing::info!(
            pattern = pattern,
            approved = approved,
            "coordination.verification"
        );
    }

    pub fn record_reputation_update(&self, agent_id: &AgentId, composite_score: f64) {
        tracing::info!(
            agent_id = ?agent_id.0,
            composite_score = composite_score,
            "coordination.reputation_update"
        );
    }

    pub fn record_governor_decision(&self, check_type: &str, allowed: bool) {
        tracing::info!(
            check_type = check_type,
            allowed = allowed,
            "coordination.governor_decision"
        );
    }

    pub fn record_resource_utilization(&self, cpu_pct: f64, mem_pct: f64, token_pct: f64) {
        tracing::info!(
            cpu_utilization = cpu_pct,
            memory_utilization = mem_pct,
            token_utilization = token_pct,
            "coordination.resource_utilization"
        );
    }
}

impl Default for CoordinationMetrics {
    fn default() -> Self {
        Self::new()
    }
}
