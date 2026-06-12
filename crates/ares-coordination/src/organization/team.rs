use ares_agent_runtime::models::{AgentId, AgentRole, TeamId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A team of agents working toward a shared goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTeam {
    pub id: TeamId,
    pub name: String,
    pub leader_id: Option<AgentId>,
    pub members: HashMap<AgentId, AgentRole>,
    pub goal: String,
    pub strategy: TeamStrategy,
    pub status: TeamStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeamStrategy {
    /// All members work in parallel on different sub-tasks.
    Parallel,
    /// Members work in a pipeline sequence.
    Pipeline,
    /// Leader delegates and coordinates.
    Hierarchical,
    /// Members compete, best result wins.
    Competitive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeamStatus {
    Forming,
    Active,
    Executing,
    Completed,
    Disbanded,
}

impl AgentTeam {
    pub fn new(name: impl Into<String>, goal: impl Into<String>) -> Self {
        Self {
            id: TeamId::new(),
            name: name.into(),
            leader_id: None,
            members: HashMap::new(),
            goal: goal.into(),
            strategy: TeamStrategy::Parallel,
            status: TeamStatus::Forming,
        }
    }

    pub fn with_strategy(mut self, strategy: TeamStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn set_leader(&mut self, agent_id: AgentId) {
        self.leader_id = Some(agent_id);
    }

    pub fn add_member(&mut self, agent_id: AgentId, role: AgentRole) {
        self.members.insert(agent_id, role);
    }

    pub fn remove_member(&mut self, agent_id: &AgentId) -> bool {
        if self.leader_id.as_ref() == Some(agent_id) {
            self.leader_id = None;
        }
        self.members.remove(agent_id).is_some()
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    pub fn has_member(&self, agent_id: &AgentId) -> bool {
        self.members.contains_key(agent_id)
    }

    pub fn get_members_by_role(&self, role: &AgentRole) -> Vec<AgentId> {
        self.members
            .iter()
            .filter(|(_, r)| *r == role)
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn is_complete(&self) -> bool {
        self.leader_id.is_some() && !self.members.is_empty()
    }

    pub fn set_status(&mut self, status: TeamStatus) {
        self.status = status;
    }

    pub fn activate(&mut self) {
        self.status = TeamStatus::Active;
    }

    pub fn disband(&mut self) {
        self.status = TeamStatus::Disbanded;
        self.members.clear();
        self.leader_id = None;
    }
}
