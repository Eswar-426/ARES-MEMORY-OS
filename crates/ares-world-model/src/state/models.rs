use ares_core::WorldStateId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Complete snapshot of the system's current reality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub id: WorldStateId,
    pub goals: Vec<WorldGoal>,
    pub resources: Vec<WorldResource>,
    pub active_agents: Vec<WorldAgent>,
    pub constraints: Vec<WorldConstraint>,
    pub snapshot_at: DateTime<Utc>,
}

impl WorldState {
    /// Create a new empty world state snapshot.
    pub fn new() -> Self {
        Self {
            id: WorldStateId::new(),
            goals: Vec::new(),
            resources: Vec::new(),
            active_agents: Vec::new(),
            constraints: Vec::new(),
            snapshot_at: Utc::now(),
        }
    }

    /// Total available budget from all budget-type resources.
    pub fn total_budget(&self) -> f64 {
        self.resources
            .iter()
            .filter(|r| r.resource_type == ResourceType::Budget)
            .map(|r| r.available)
            .sum()
    }

    /// Number of agents in a ready/idle state.
    pub fn available_agent_count(&self) -> usize {
        self.active_agents
            .iter()
            .filter(|a| a.status == "ready" || a.status == "idle")
            .count()
    }

    /// Average success rate across all agents.
    pub fn average_agent_success_rate(&self) -> f64 {
        if self.active_agents.is_empty() {
            return 0.0;
        }
        let total: f64 = self.active_agents.iter().map(|a| a.success_rate).sum();
        total / self.active_agents.len() as f64
    }

    /// Check if any hard constraint is violated.
    pub fn has_violated_constraints(&self) -> bool {
        self.constraints
            .iter()
            .any(|c| c.is_hard && c.is_violated())
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new()
    }
}

/// A goal as seen by the world state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldGoal {
    pub id: String,
    pub title: String,
    pub priority: String,
    pub status: String,
}

/// A resource tracked by the world state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldResource {
    pub name: String,
    pub resource_type: ResourceType,
    pub available: f64,
    pub capacity: f64,
}

impl WorldResource {
    /// Utilization ratio (0.0..=1.0).
    pub fn utilization(&self) -> f64 {
        if self.capacity <= 0.0 {
            return 0.0;
        }
        ((self.capacity - self.available) / self.capacity).clamp(0.0, 1.0)
    }
}

/// Type of resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Budget,
    Compute,
    Memory,
    TokenBudget,
    ApiCalls,
    Custom(String),
}

/// An agent tracked by the world state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldAgent {
    pub id: String,
    pub name: String,
    pub role: String,
    pub status: String,
    pub success_rate: f64,
}

/// A constraint on the world state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConstraint {
    pub name: String,
    pub constraint_type: ConstraintType,
    pub value: f64,
    pub current: f64,
    pub is_hard: bool,
}

impl WorldConstraint {
    /// Whether the constraint is currently violated.
    pub fn is_violated(&self) -> bool {
        match self.constraint_type {
            ConstraintType::MaxBudget => self.current > self.value,
            ConstraintType::MaxDuration => self.current > self.value,
            ConstraintType::MinQuality => self.current < self.value,
            ConstraintType::MaxAgents => self.current > self.value,
            ConstraintType::MaxRetries => self.current > self.value,
            ConstraintType::Custom(_) => false,
        }
    }

    /// How much slack remains before violation (positive = safe, negative = violated).
    pub fn slack(&self) -> f64 {
        match self.constraint_type {
            ConstraintType::MinQuality => self.current - self.value,
            _ => self.value - self.current,
        }
    }
}

/// Type of constraint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintType {
    MaxBudget,
    MaxDuration,
    MinQuality,
    MaxAgents,
    MaxRetries,
    Custom(String),
}

/// Diff between two world states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldStateDiff {
    pub goals_added: usize,
    pub goals_removed: usize,
    pub resources_changed: usize,
    pub agents_changed: usize,
    pub constraints_changed: usize,
    pub budget_delta: f64,
    pub agent_count_delta: i64,
}
