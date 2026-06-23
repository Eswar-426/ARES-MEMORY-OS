use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a node in the cluster.
#[derive(utoipa::ToSchema, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn local() -> Self {
        // Deterministic ID for the single local node
        Self(Uuid::nil())
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::local()
    }
}

/// State of a worker node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeState {
    Initializing,
    Ready,
    Active,
    Draining,
    Offline,
}

/// A worker node in the ARES cluster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerNode {
    pub id: NodeId,
    pub name: String,
    pub address: String,
    pub state: NodeState,
    pub capacity_agents: u32,
    pub active_agents: u32,
    pub last_heartbeat: i64,
    pub joined_at: i64,
}

impl WorkerNode {
    /// Create the local single-node instance.
    pub fn local(capacity: u32) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: NodeId::local(),
            name: "local".into(),
            address: "127.0.0.1".into(),
            state: NodeState::Active,
            capacity_agents: capacity,
            active_agents: 0,
            last_heartbeat: now,
            joined_at: now,
        }
    }

    pub fn is_available(&self) -> bool {
        self.state == NodeState::Active && self.active_agents < self.capacity_agents
    }

    pub fn available_slots(&self) -> u32 {
        self.capacity_agents.saturating_sub(self.active_agents)
    }

    pub fn allocate_agent(&mut self) -> Result<(), String> {
        if self.active_agents >= self.capacity_agents {
            return Err("Node at capacity".into());
        }
        self.active_agents += 1;
        Ok(())
    }

    pub fn release_agent(&mut self) {
        if self.active_agents > 0 {
            self.active_agents -= 1;
        }
    }

    pub fn heartbeat(&mut self) {
        self.last_heartbeat = chrono::Utc::now().timestamp();
    }

    pub fn is_heartbeat_stale(&self, timeout_secs: i64) -> bool {
        let now = chrono::Utc::now().timestamp();
        now - self.last_heartbeat > timeout_secs
    }
}
