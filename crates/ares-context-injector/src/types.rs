#![allow(deprecated)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    pub decisions: Vec<Decision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub author: String,
    pub message: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitContext {
    pub commits: Vec<GitCommit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstContext {
    pub nodes: Vec<GraphNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborContext {
    pub nodes: Vec<GraphNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipContext {
    pub owners: Vec<GraphNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureContext {
    pub docs: Vec<GraphNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementContext {
    pub reqs: Vec<GraphNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ContextPackage {
    pub project_id: String,
    pub original_prompt: String,
    pub assembled_prompt: String,
    pub estimated_tokens: usize,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PromptSection {
    pub priority: u32,
    pub title: String,
    pub content: String,
    pub item_count: usize,
    pub items: Vec<String>,
}
