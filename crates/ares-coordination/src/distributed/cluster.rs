use std::collections::HashMap;

use super::node::{NodeId, NodeState, WorkerNode};

/// Cluster membership management.
pub struct ClusterMembership {
    nodes: HashMap<NodeId, WorkerNode>,
    leader_id: Option<NodeId>,
}

impl ClusterMembership {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            leader_id: None,
        }
    }

    /// Create a single-node cluster (current mode).
    pub fn single_node(capacity: u32) -> Self {
        let mut membership = Self::new();
        let node = WorkerNode::local(capacity);
        let node_id = node.id;
        membership.nodes.insert(node_id, node);
        membership.leader_id = Some(node_id);
        membership
    }

    /// Add a node to the cluster.
    pub fn add_node(&mut self, node: WorkerNode) {
        let id = node.id;
        self.nodes.insert(id, node);
        // First node becomes leader
        if self.leader_id.is_none() {
            self.leader_id = Some(id);
        }
    }

    /// Remove a node from the cluster.
    pub fn remove_node(&mut self, node_id: &NodeId) -> Option<WorkerNode> {
        let node = self.nodes.remove(node_id);
        if self.leader_id.as_ref() == Some(node_id) {
            self.leader_id = self.nodes.keys().next().copied();
        }
        node
    }

    /// Get the current leader.
    pub fn leader(&self) -> Option<&WorkerNode> {
        self.leader_id.and_then(|id| self.nodes.get(&id))
    }

    /// Get a node by ID.
    pub fn get_node(&self, node_id: &NodeId) -> Option<&WorkerNode> {
        self.nodes.get(node_id)
    }

    /// Get a mutable node.
    pub fn get_node_mut(&mut self, node_id: &NodeId) -> Option<&mut WorkerNode> {
        self.nodes.get_mut(node_id)
    }

    /// Get all available nodes.
    pub fn available_nodes(&self) -> Vec<&WorkerNode> {
        self.nodes.values().filter(|n| n.is_available()).collect()
    }

    /// Get node count.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Check if this is a single-node cluster.
    pub fn is_single_node(&self) -> bool {
        self.nodes.len() <= 1
    }

    /// Get total capacity across all nodes.
    pub fn total_capacity(&self) -> u32 {
        self.nodes.values().map(|n| n.capacity_agents).sum()
    }

    /// Get total active agents across all nodes.
    pub fn total_active(&self) -> u32 {
        self.nodes.values().map(|n| n.active_agents).sum()
    }
}

impl Default for ClusterMembership {
    fn default() -> Self {
        Self::single_node(16)
    }
}

/// Leader election (placeholder for future distributed consensus).
pub struct LeaderElection;

impl LeaderElection {
    pub fn new() -> Self {
        Self
    }

    /// In single-node mode, the local node is always the leader.
    pub fn elect_leader(membership: &ClusterMembership) -> Option<NodeId> {
        membership.leader_id
    }

    /// Check if a node is the leader.
    pub fn is_leader(membership: &ClusterMembership, node_id: &NodeId) -> bool {
        membership.leader_id.as_ref() == Some(node_id)
    }
}

impl Default for LeaderElection {
    fn default() -> Self {
        Self::new()
    }
}

/// Heartbeat tracking for node health.
pub struct Heartbeat {
    timeout_secs: i64,
}

impl Heartbeat {
    pub fn new(timeout_secs: i64) -> Self {
        Self { timeout_secs }
    }

    /// Check for stale nodes and mark them as offline.
    pub fn check_health(&self, membership: &mut ClusterMembership) -> Vec<NodeId> {
        let mut stale = Vec::new();
        for (id, node) in membership.nodes.iter_mut() {
            if node.is_heartbeat_stale(self.timeout_secs) && node.state == NodeState::Active {
                node.state = NodeState::Offline;
                stale.push(*id);
            }
        }
        stale
    }

    /// Record a heartbeat for a node.
    pub fn record_heartbeat(&self, membership: &mut ClusterMembership, node_id: &NodeId) {
        if let Some(node) = membership.get_node_mut(node_id) {
            node.heartbeat();
            if node.state == NodeState::Offline {
                node.state = NodeState::Active;
            }
        }
    }
}

impl Default for Heartbeat {
    fn default() -> Self {
        Self::new(30)
    }
}
