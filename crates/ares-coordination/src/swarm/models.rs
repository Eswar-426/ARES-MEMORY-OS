use ares_agent_runtime::models::AgentId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique swarm execution identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SwarmId(pub Uuid);

impl SwarmId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for SwarmId {
    fn default() -> Self {
        Self::new()
    }
}

/// State of a swarm execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwarmState {
    Initializing,
    Running,
    Collecting,
    Selecting,
    Completed,
    Failed,
    Terminated,
}

/// Individual agent result within a swarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmAgentResult {
    pub agent_id: AgentId,
    pub output: String,
    pub quality_score: f64,
    pub latency_ms: u64,
    pub success: bool,
}

/// Aggregated result of a swarm execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmResult {
    pub best_result: Option<SwarmAgentResult>,
    pub all_results: Vec<SwarmAgentResult>,
    pub success_rate: f64,
    pub avg_quality: f64,
    pub total_latency_ms: u64,
}

impl SwarmResult {
    pub fn from_results(results: Vec<SwarmAgentResult>) -> Self {
        let total = results.len() as f64;
        let successes = results.iter().filter(|r| r.success).count() as f64;
        let avg_quality = if total > 0.0 {
            results.iter().map(|r| r.quality_score).sum::<f64>() / total
        } else {
            0.0
        };
        let total_latency = results
            .iter()
            .map(|r| r.latency_ms)
            .max()
            .unwrap_or_default();

        let best = results
            .iter()
            .filter(|r| r.success)
            .max_by(|a, b| {
                a.quality_score
                    .partial_cmp(&b.quality_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned();

        Self {
            best_result: best,
            all_results: results,
            success_rate: if total > 0.0 { successes / total } else { 0.0 },
            avg_quality,
            total_latency_ms: total_latency,
        }
    }
}

/// A swarm execution record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmExecution {
    pub id: SwarmId,
    pub task_description: String,
    pub agents: Vec<AgentId>,
    pub state: SwarmState,
    pub results: Vec<SwarmAgentResult>,
    pub best_agent: Option<AgentId>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

impl SwarmExecution {
    pub fn new(task_description: impl Into<String>, agents: Vec<AgentId>) -> Self {
        Self {
            id: SwarmId::new(),
            task_description: task_description.into(),
            agents,
            state: SwarmState::Initializing,
            results: Vec::new(),
            best_agent: None,
            created_at: chrono::Utc::now().timestamp(),
            completed_at: None,
        }
    }

    pub fn add_result(&mut self, result: SwarmAgentResult) {
        self.results.push(result);
    }

    pub fn finalize(&mut self) -> SwarmResult {
        let result = SwarmResult::from_results(self.results.clone());
        self.best_agent = result.best_result.as_ref().map(|r| r.agent_id);
        self.state = SwarmState::Completed;
        self.completed_at = Some(chrono::Utc::now().timestamp());
        result
    }

    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    pub fn result_count(&self) -> usize {
        self.results.len()
    }
}
