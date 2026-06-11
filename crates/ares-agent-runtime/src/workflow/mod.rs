use crate::models::{AgentRole, TaskId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissionNode {
    pub id: TaskId,
    pub name: String,
    pub role: AgentRole,
    pub payload: String, // E.g. prompt or instructions
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissionEdge {
    pub from: TaskId,
    pub to: TaskId,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MissionDag {
    pub nodes: HashMap<TaskId, MissionNode>,
    pub edges: Vec<MissionEdge>,
}

impl MissionDag {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: MissionNode) {
        self.nodes.insert(node.id, node);
    }

    pub fn add_edge(&mut self, edge: MissionEdge) {
        self.edges.push(edge);
    }

    pub fn get_roots(&self) -> Vec<TaskId> {
        let mut has_incoming = HashSet::new();
        for edge in &self.edges {
            has_incoming.insert(edge.to);
        }

        self.nodes
            .keys()
            .filter(|id| !has_incoming.contains(id))
            .cloned()
            .collect()
    }

    pub fn get_children(&self, parent: &TaskId) -> Vec<TaskId> {
        self.edges
            .iter()
            .filter(|e| &e.from == parent)
            .map(|e| e.to)
            .collect()
    }
}

pub struct MissionExecutor {
    pub dag: MissionDag,
    pub completed_nodes: HashSet<TaskId>,
    pub running_nodes: HashSet<TaskId>,
}

impl MissionExecutor {
    pub fn new(dag: MissionDag) -> Self {
        Self {
            dag,
            completed_nodes: HashSet::new(),
            running_nodes: HashSet::new(),
        }
    }

    pub fn get_ready_nodes(&self) -> Vec<MissionNode> {
        let mut ready = Vec::new();
        for (id, node) in &self.dag.nodes {
            if self.completed_nodes.contains(id) || self.running_nodes.contains(id) {
                continue;
            }

            // A node is ready if all its parents are completed
            let mut all_parents_completed = true;
            for edge in &self.dag.edges {
                if &edge.to == id && !self.completed_nodes.contains(&edge.from) {
                    all_parents_completed = false;
                    break;
                }
            }

            if all_parents_completed {
                ready.push(node.clone());
            }
        }
        ready
    }

    pub fn mark_running(&mut self, id: TaskId) {
        self.running_nodes.insert(id);
    }

    pub fn mark_completed(&mut self, id: TaskId) {
        self.running_nodes.remove(&id);
        self.completed_nodes.insert(id);
    }
}
