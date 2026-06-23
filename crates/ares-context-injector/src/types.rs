use ares_core::{Decision, GraphNode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TokenBudget {
    Small = 4000,
    #[default]
    Medium = 8000,
    Large = 16000,
    Maximum = 32000,
}

impl TokenBudget {
    pub fn as_usize(&self) -> usize {
        *self as usize
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
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
