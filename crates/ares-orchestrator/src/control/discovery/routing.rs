use crate::control::workers::models::{Worker, WorkerStatus};
use crate::control::workers::repository::WorkerRepository;
use ares_core::AresError;
use std::sync::Arc;

pub struct DiscoveryService {
    worker_repo: Arc<WorkerRepository>,
}

impl DiscoveryService {
    pub fn new(worker_repo: Arc<WorkerRepository>) -> Self {
        Self { worker_repo }
    }

    pub fn find_workers_by_capability(&self, name: &str, version: Option<&str>) -> Result<Vec<Worker>, AresError> {
        let workers = self.worker_repo.list()?;
        let matched: Vec<Worker> = workers.into_iter()
            .filter(|w| w.status == WorkerStatus::Online || w.status == WorkerStatus::Busy)
            .filter(|w| {
                w.capabilities.iter().any(|cap| {
                    if cap.name == name {
                        if let Some(v) = version {
                            cap.version == v
                        } else {
                            true
                        }
                    } else {
                        false
                    }
                })
            })
            .collect();
        
        Ok(matched)
    }
}
