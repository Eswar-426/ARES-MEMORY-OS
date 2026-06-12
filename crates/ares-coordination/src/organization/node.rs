use ares_agent_runtime::models::{AgentId, AgentRole, TeamId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A node in the organizational tree representing a single agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentNode {
    pub agent_id: AgentId,
    pub role: AgentRole,
    pub team_id: Option<TeamId>,
    pub parent_id: Option<AgentId>,
    pub children: HashSet<AgentId>,
    pub capabilities: Vec<String>,
    pub status: NodeStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    Active,
    Idle,
    Busy,
    Suspended,
    Terminated,
}

impl AgentNode {
    pub fn new(agent_id: AgentId, role: AgentRole) -> Self {
        Self {
            agent_id,
            role,
            team_id: None,
            parent_id: None,
            children: HashSet::new(),
            capabilities: Vec::new(),
            status: NodeStatus::Active,
        }
    }

    pub fn with_team(mut self, team_id: TeamId) -> Self {
        self.team_id = Some(team_id);
        self
    }

    pub fn with_parent(mut self, parent_id: AgentId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn add_child(&mut self, child_id: AgentId) {
        self.children.insert(child_id);
    }

    pub fn remove_child(&mut self, child_id: &AgentId) -> bool {
        self.children.remove(child_id)
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }

    pub fn set_status(&mut self, status: NodeStatus) {
        self.status = status;
    }
}
