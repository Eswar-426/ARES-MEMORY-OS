use ares_agent_runtime::models::TaskId;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::models::{ReservationId, ResourceCapacity, ResourceRequirements, ResourceUtilization};

/// Resource reservation record.
#[derive(Debug, Clone)]
struct Reservation {
    pub id: ReservationId,
    pub _task_id: TaskId,
    pub requirements: ResourceRequirements,
    pub _created_at: i64,
}

/// Pre-delegation resource manager. Ensures resources are available before
/// task assignment, preventing "assign then fail" patterns.
pub struct CoordinationResourceManager {
    capacity: ResourceCapacity,
    cpu_used: AtomicU32,
    memory_used_mb: AtomicU64,
    gpu_used: AtomicU32,
    tokens_consumed: AtomicU64,
    tool_slots_used: AtomicU32,
    network_slots_used: AtomicU32,
    reservations: Arc<RwLock<HashMap<ReservationId, Reservation>>>,
}

impl CoordinationResourceManager {
    pub fn new(capacity: ResourceCapacity) -> Self {
        Self {
            capacity,
            cpu_used: AtomicU32::new(0),
            memory_used_mb: AtomicU64::new(0),
            gpu_used: AtomicU32::new(0),
            tokens_consumed: AtomicU64::new(0),
            tool_slots_used: AtomicU32::new(0),
            network_slots_used: AtomicU32::new(0),
            reservations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if the required resources are available without reserving.
    pub fn check_availability(&self, requirements: &ResourceRequirements) -> bool {
        let cpu_available = self.capacity.cpu_slots - self.cpu_used.load(Ordering::Relaxed)
            >= requirements.cpu_slots;
        let mem_available = self.capacity.memory_mb - self.memory_used_mb.load(Ordering::Relaxed)
            >= requirements.memory_mb;
        let gpu_available = self.capacity.gpu_slots - self.gpu_used.load(Ordering::Relaxed)
            >= requirements.gpu_slots;
        let tokens_available = self.capacity.token_budget
            - self.tokens_consumed.load(Ordering::Relaxed)
            >= requirements.token_budget;
        let tools_available = self.capacity.tool_slots
            - self.tool_slots_used.load(Ordering::Relaxed)
            >= requirements.tool_slots;
        let network_available = self.capacity.network_slots
            - self.network_slots_used.load(Ordering::Relaxed)
            >= requirements.network_slots;

        cpu_available
            && mem_available
            && gpu_available
            && tokens_available
            && tools_available
            && network_available
    }

    /// Reserve resources for a task. Returns ReservationId on success.
    pub async fn reserve(
        &self,
        task_id: TaskId,
        requirements: ResourceRequirements,
    ) -> Result<ReservationId, String> {
        if !self.check_availability(&requirements) {
            return Err("Insufficient resources".into());
        }

        // Atomically reserve
        self.cpu_used
            .fetch_add(requirements.cpu_slots, Ordering::Relaxed);
        self.memory_used_mb
            .fetch_add(requirements.memory_mb, Ordering::Relaxed);
        self.gpu_used
            .fetch_add(requirements.gpu_slots, Ordering::Relaxed);
        self.tokens_consumed
            .fetch_add(requirements.token_budget, Ordering::Relaxed);
        self.tool_slots_used
            .fetch_add(requirements.tool_slots, Ordering::Relaxed);
        self.network_slots_used
            .fetch_add(requirements.network_slots, Ordering::Relaxed);

        let reservation = Reservation {
            id: ReservationId::new(),
            _task_id: task_id,
            requirements,
            _created_at: chrono::Utc::now().timestamp(),
        };

        let id = reservation.id;
        self.reservations.write().await.insert(id, reservation);
        Ok(id)
    }

    /// Release a reservation, freeing the resources.
    pub async fn release(&self, reservation_id: &ReservationId) -> Result<(), String> {
        let mut reservations = self.reservations.write().await;
        if let Some(reservation) = reservations.remove(reservation_id) {
            self.cpu_used
                .fetch_sub(reservation.requirements.cpu_slots, Ordering::Relaxed);
            self.memory_used_mb
                .fetch_sub(reservation.requirements.memory_mb, Ordering::Relaxed);
            self.gpu_used
                .fetch_sub(reservation.requirements.gpu_slots, Ordering::Relaxed);
            self.tokens_consumed
                .fetch_sub(reservation.requirements.token_budget, Ordering::Relaxed);
            self.tool_slots_used
                .fetch_sub(reservation.requirements.tool_slots, Ordering::Relaxed);
            self.network_slots_used
                .fetch_sub(reservation.requirements.network_slots, Ordering::Relaxed);
            Ok(())
        } else {
            Err(format!("Reservation {:?} not found", reservation_id))
        }
    }

    /// Consume tokens from the budget (non-reservable, direct debit).
    pub fn consume_tokens(&self, tokens: u64) -> Result<(), String> {
        let current = self.tokens_consumed.load(Ordering::Relaxed);
        if current + tokens > self.capacity.token_budget {
            return Err("Token budget exceeded".into());
        }
        self.tokens_consumed.fetch_add(tokens, Ordering::Relaxed);
        Ok(())
    }

    /// Get current resource utilization snapshot.
    pub async fn get_utilization(&self) -> ResourceUtilization {
        ResourceUtilization {
            cpu_used: self.cpu_used.load(Ordering::Relaxed),
            cpu_total: self.capacity.cpu_slots,
            memory_used_mb: self.memory_used_mb.load(Ordering::Relaxed),
            memory_total_mb: self.capacity.memory_mb,
            gpu_used: self.gpu_used.load(Ordering::Relaxed),
            gpu_total: self.capacity.gpu_slots,
            tokens_consumed: self.tokens_consumed.load(Ordering::Relaxed),
            token_budget: self.capacity.token_budget,
            tool_slots_used: self.tool_slots_used.load(Ordering::Relaxed),
            tool_slots_total: self.capacity.tool_slots,
            network_slots_used: self.network_slots_used.load(Ordering::Relaxed),
            network_slots_total: self.capacity.network_slots,
            active_reservations: self.reservations.read().await.len(),
        }
    }

    /// Get the count of active reservations.
    pub async fn reservation_count(&self) -> usize {
        self.reservations.read().await.len()
    }
}

impl Default for CoordinationResourceManager {
    fn default() -> Self {
        Self::new(ResourceCapacity::default())
    }
}
