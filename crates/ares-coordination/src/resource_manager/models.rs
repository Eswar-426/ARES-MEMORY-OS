use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a resource reservation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReservationId(pub Uuid);

impl ReservationId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for ReservationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource requirements for a task or agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_slots: u32,
    pub memory_mb: u64,
    pub gpu_slots: u32,
    pub token_budget: u64,
    pub tool_slots: u32,
    pub network_slots: u32,
}

impl ResourceRequirements {
    pub fn minimal() -> Self {
        Self {
            cpu_slots: 1,
            memory_mb: 64,
            gpu_slots: 0,
            token_budget: 1000,
            tool_slots: 1,
            network_slots: 0,
        }
    }

    pub fn standard() -> Self {
        Self {
            cpu_slots: 2,
            memory_mb: 256,
            gpu_slots: 0,
            token_budget: 10000,
            tool_slots: 2,
            network_slots: 1,
        }
    }

    pub fn heavy() -> Self {
        Self {
            cpu_slots: 4,
            memory_mb: 1024,
            gpu_slots: 1,
            token_budget: 50000,
            tool_slots: 4,
            network_slots: 2,
        }
    }
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self::minimal()
    }
}

/// Current resource utilization snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub cpu_used: u32,
    pub cpu_total: u32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub gpu_used: u32,
    pub gpu_total: u32,
    pub tokens_consumed: u64,
    pub token_budget: u64,
    pub tool_slots_used: u32,
    pub tool_slots_total: u32,
    pub network_slots_used: u32,
    pub network_slots_total: u32,
    pub active_reservations: usize,
}

impl ResourceUtilization {
    pub fn cpu_utilization(&self) -> f64 {
        if self.cpu_total == 0 {
            return 0.0;
        }
        self.cpu_used as f64 / self.cpu_total as f64
    }

    pub fn memory_utilization(&self) -> f64 {
        if self.memory_total_mb == 0 {
            return 0.0;
        }
        self.memory_used_mb as f64 / self.memory_total_mb as f64
    }

    pub fn token_utilization(&self) -> f64 {
        if self.token_budget == 0 {
            return 0.0;
        }
        self.tokens_consumed as f64 / self.token_budget as f64
    }

    pub fn is_overloaded(&self) -> bool {
        self.cpu_utilization() > 0.9
            || self.memory_utilization() > 0.9
            || self.token_utilization() > 0.95
    }
}

/// Capacity limits for the coordination resource pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapacity {
    pub cpu_slots: u32,
    pub memory_mb: u64,
    pub gpu_slots: u32,
    pub token_budget: u64,
    pub tool_slots: u32,
    pub network_slots: u32,
}

impl Default for ResourceCapacity {
    fn default() -> Self {
        Self {
            cpu_slots: 16,
            memory_mb: 4096,
            gpu_slots: 0,
            token_budget: 1_000_000,
            tool_slots: 8,
            network_slots: 4,
        }
    }
}
