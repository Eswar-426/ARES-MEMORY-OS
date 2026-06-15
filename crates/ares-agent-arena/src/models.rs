use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    Baseline,
    ContextAware,
    EnhancedContext,
    Planner,
}

impl ToString for AgentType {
    fn to_string(&self) -> String {
        match self {
            AgentType::Baseline => "Baseline".to_string(),
            AgentType::ContextAware => "ContextAware".to_string(),
            AgentType::EnhancedContext => "EnhancedContext".to_string(),
            AgentType::Planner => "Planner".to_string(),
        }
    }
}

impl Hash for AgentType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArenaTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub expected_files: Vec<String>,
    pub expected_components: Vec<String>,
    pub expected_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRunResult {
    pub task_id: String,
    pub agent_type: AgentType,
    pub response: String,
    pub latency_ms: u64,
    pub context_nodes_used: usize,
    pub retrieved_files: Vec<String>,
    pub retrieved_components: Vec<String>,
    pub precision_score: f32,
    pub recall_score: f32,
    pub confidence_score: f32,
}
