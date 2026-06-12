use ares_agent_runtime::models::AgentId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::models::{SwarmAgentResult, SwarmExecution, SwarmId, SwarmResult, SwarmState};
use super::strategies::SwarmStrategy;
use crate::governor::SafetyGovernor;

/// Engine for coordinating swarm executions.
pub struct SwarmEngine {
    executions: Arc<RwLock<HashMap<SwarmId, SwarmExecution>>>,
}

impl SwarmEngine {
    pub fn new() -> Self {
        Self {
            executions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Launch a swarm execution.
    pub async fn launch_swarm(
        &self,
        strategy: &SwarmStrategy,
        task: impl Into<String>,
        agents: Vec<AgentId>,
        governor: Option<&SafetyGovernor>,
    ) -> Result<SwarmId, String> {
        if agents.len() < strategy.recommended_min_agents() {
            return Err(format!(
                "Strategy {:?} requires at least {} agents, got {}",
                strategy,
                strategy.recommended_min_agents(),
                agents.len()
            ));
        }

        if let Some(gov) = governor {
            let decision = gov.check_swarm_launch(agents.len() as u32).await;
            if decision.is_denied() {
                return Err(format!("Governor denied swarm launch: {:?}", decision));
            }
        }

        let mut execution = SwarmExecution::new(task, agents);
        execution.state = SwarmState::Running;
        let id = execution.id;
        self.executions.write().await.insert(id, execution);
        Ok(id)
    }

    /// Submit an agent's result for a swarm execution.
    pub async fn submit_result(
        &self,
        swarm_id: &SwarmId,
        result: SwarmAgentResult,
    ) -> Result<(), String> {
        let mut executions = self.executions.write().await;
        if let Some(execution) = executions.get_mut(swarm_id) {
            if !execution.agents.contains(&result.agent_id) {
                return Err("Agent not part of this swarm".into());
            }
            execution.add_result(result);
            Ok(())
        } else {
            Err(format!("Swarm {:?} not found", swarm_id))
        }
    }

    /// Collect and finalize results, selecting the best.
    pub async fn collect_results(&self, swarm_id: &SwarmId) -> Result<SwarmResult, String> {
        let mut executions = self.executions.write().await;
        if let Some(execution) = executions.get_mut(swarm_id) {
            execution.state = SwarmState::Collecting;
            let result = execution.finalize();
            Ok(result)
        } else {
            Err(format!("Swarm {:?} not found", swarm_id))
        }
    }

    /// Terminate a running swarm.
    pub async fn terminate_swarm(&self, swarm_id: &SwarmId) -> Result<(), String> {
        let mut executions = self.executions.write().await;
        if let Some(execution) = executions.get_mut(swarm_id) {
            execution.state = SwarmState::Terminated;
            execution.completed_at = Some(chrono::Utc::now().timestamp());
            Ok(())
        } else {
            Err(format!("Swarm {:?} not found", swarm_id))
        }
    }

    /// Get a swarm execution.
    pub async fn get_execution(&self, swarm_id: &SwarmId) -> Option<SwarmExecution> {
        self.executions.read().await.get(swarm_id).cloned()
    }

    /// Get total swarm count.
    pub async fn swarm_count(&self) -> usize {
        self.executions.read().await.len()
    }
}

impl Default for SwarmEngine {
    fn default() -> Self {
        Self::new()
    }
}
