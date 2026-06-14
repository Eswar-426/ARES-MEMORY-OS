use ares_core::{Decision, GraphNode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TokenBudget {
    Small = 4000,
    Medium = 8000,
    Large = 16000,
    Maximum = 32000,
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self::Medium
    }
}

impl TokenBudget {
    pub fn as_usize(&self) -> usize {
        *self as usize
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPackage {
    pub project_id: String,
    pub original_prompt: String,
    pub architecture_nodes: Vec<GraphNode>,
    pub decisions: Vec<Decision>,
    pub bugs: Vec<GraphNode>,
    pub memories: Vec<GraphNode>,
    pub assembled_prompt: String,
    pub estimated_tokens: usize,
}
