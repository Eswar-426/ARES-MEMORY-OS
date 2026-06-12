use ares_agent_runtime::models::{AgentId, AgentRole, OrganizationId, TeamId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::node::{AgentNode, NodeStatus};
use super::team::AgentTeam;

/// Organizational hierarchy — a tree of teams and agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHierarchy {
    pub id: OrganizationId,
    pub name: String,
    pub nodes: HashMap<AgentId, AgentNode>,
    pub teams: HashMap<TeamId, AgentTeam>,
    pub root_id: Option<AgentId>,
}

impl AgentHierarchy {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: OrganizationId::new(),
            name: name.into(),
            nodes: HashMap::new(),
            teams: HashMap::new(),
            root_id: None,
        }
    }

    /// Add a node to the hierarchy. If it's the first node, it becomes root.
    pub fn add_node(&mut self, node: AgentNode) {
        let id = node.agent_id;
        if self.nodes.is_empty() {
            self.root_id = Some(id);
        }
        self.nodes.insert(id, node);
    }

    /// Set a parent-child relationship between two existing nodes.
    pub fn set_parent(&mut self, child_id: AgentId, parent_id: AgentId) -> Result<(), String> {
        if !self.nodes.contains_key(&child_id) {
            return Err(format!("Child node {:?} not found", child_id));
        }
        if !self.nodes.contains_key(&parent_id) {
            return Err(format!("Parent node {:?} not found", parent_id));
        }

        // Update child's parent
        if let Some(child) = self.nodes.get_mut(&child_id) {
            child.parent_id = Some(parent_id);
        }

        // Update parent's children
        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            parent.add_child(child_id);
        }

        Ok(())
    }

    /// Add a team to the hierarchy.
    pub fn add_team(&mut self, team: AgentTeam) {
        self.teams.insert(team.id, team);
    }

    /// Get the root node of the hierarchy.
    pub fn get_root(&self) -> Option<&AgentNode> {
        self.root_id.and_then(|id| self.nodes.get(&id))
    }

    /// Get a node by agent ID.
    pub fn get_node(&self, agent_id: &AgentId) -> Option<&AgentNode> {
        self.nodes.get(agent_id)
    }

    /// Get a mutable node by agent ID.
    pub fn get_node_mut(&mut self, agent_id: &AgentId) -> Option<&mut AgentNode> {
        self.nodes.get_mut(agent_id)
    }

    /// Get all direct children of a node.
    pub fn get_children(&self, agent_id: &AgentId) -> Vec<&AgentNode> {
        self.nodes
            .get(agent_id)
            .map(|node| {
                node.children
                    .iter()
                    .filter_map(|child_id| self.nodes.get(child_id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all descendants (recursive) of a node.
    pub fn get_descendants(&self, agent_id: &AgentId) -> Vec<&AgentNode> {
        let mut result = Vec::new();
        let mut stack = vec![*agent_id];

        while let Some(current) = stack.pop() {
            if let Some(node) = self.nodes.get(&current) {
                if current != *agent_id {
                    result.push(node);
                }
                for child_id in &node.children {
                    stack.push(*child_id);
                }
            }
        }
        result
    }

    /// Get the depth of a node in the hierarchy (root = 0).
    pub fn get_depth(&self, agent_id: &AgentId) -> usize {
        let mut depth = 0;
        let mut current = *agent_id;
        while let Some(node) = self.nodes.get(&current) {
            if let Some(parent) = node.parent_id {
                depth += 1;
                current = parent;
            } else {
                break;
            }
        }
        depth
    }

    /// Get all nodes with a specific role.
    pub fn get_nodes_by_role(&self, role: &AgentRole) -> Vec<&AgentNode> {
        self.nodes
            .values()
            .filter(|node| node.role == *role)
            .collect()
    }

    /// Get all active nodes.
    pub fn get_active_nodes(&self) -> Vec<&AgentNode> {
        self.nodes
            .values()
            .filter(|node| node.status == NodeStatus::Active || node.status == NodeStatus::Idle)
            .collect()
    }

    /// Get a team by ID.
    pub fn get_team(&self, team_id: &TeamId) -> Option<&AgentTeam> {
        self.teams.get(team_id)
    }

    /// Total number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Total number of teams.
    pub fn team_count(&self) -> usize {
        self.teams.len()
    }

    /// Remove a node and reparent its children to its parent.
    pub fn remove_node(&mut self, agent_id: &AgentId) -> Result<(), String> {
        let node = self
            .nodes
            .get(agent_id)
            .ok_or_else(|| format!("Node {:?} not found", agent_id))?
            .clone();

        // Reparent children to this node's parent
        let parent_id = node.parent_id;
        for child_id in &node.children {
            if let Some(child) = self.nodes.get_mut(child_id) {
                child.parent_id = parent_id;
            }
            if let Some(pid) = parent_id {
                if let Some(parent) = self.nodes.get_mut(&pid) {
                    parent.add_child(*child_id);
                }
            }
        }

        // Remove from parent's children list
        if let Some(pid) = parent_id {
            if let Some(parent) = self.nodes.get_mut(&pid) {
                parent.remove_child(agent_id);
            }
        }

        // If this was root, promote first child
        if self.root_id == Some(*agent_id) {
            self.root_id = node.children.iter().next().copied();
        }

        self.nodes.remove(agent_id);
        Ok(())
    }
}
