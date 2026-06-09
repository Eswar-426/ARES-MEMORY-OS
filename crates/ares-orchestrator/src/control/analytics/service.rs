use crate::control::workers::models::WorkerStatus;
use crate::control::workers::repository::WorkerRepository;
use ares_core::AresError;
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct OrchestratorAnalytics {
    pub active_workers: usize,
    pub online_workers: usize,
    pub dead_workers: usize,
    pub offline_workers: usize,
    pub queue_depth: usize,
    // Future metrics:
    pub dlq_depth: usize,
    pub active_leases: usize,
}

pub struct AnalyticsService {
    worker_repo: Arc<WorkerRepository>,
}

impl AnalyticsService {
    pub fn new(worker_repo: Arc<WorkerRepository>) -> Self {
        Self { worker_repo }
    }

    pub fn get_analytics(&self) -> Result<OrchestratorAnalytics, AresError> {
        let workers = self.worker_repo.list()?;
        
        let online = workers.iter().filter(|w| w.status == WorkerStatus::Online).count();
        let busy = workers.iter().filter(|w| w.status == WorkerStatus::Busy).count();
        let dead = workers.iter().filter(|w| w.status == WorkerStatus::Dead).count();
        let offline = workers.iter().filter(|w| w.status == WorkerStatus::Offline).count();

        Ok(OrchestratorAnalytics {
            active_workers: busy,
            online_workers: online,
            dead_workers: dead,
            offline_workers: offline,
            queue_depth: 0, // Hook up to queue repository when ready
            dlq_depth: 0, // Hook up to DLQ repository when ready
            active_leases: 0, // Hook up to leases repository when ready
        })
    }
}
