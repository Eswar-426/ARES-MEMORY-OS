use super::{dto::*, mapper::WorkerMapper, models::*, repository::WorkerRepository};
use ares_core::AresError;
use chrono::Utc;

pub struct WorkerService {
    repo: WorkerRepository,
}

impl WorkerService {
    pub fn new(repo: WorkerRepository) -> Self {
        Self { repo }
    }

    pub fn register_worker(&self, req: WorkerRegistrationRequest) -> Result<Worker, AresError> {
        let worker = WorkerMapper::from_registration(req);
        self.repo.register(&worker)?;
        
        // Let's immediately transition it to Online after registering.
        let now = Utc::now().to_rfc3339();
        let resources_json = serde_json::to_string(&worker.resources).unwrap_or_default();
        self.repo.update_status(&worker.id, &WorkerStatus::Online, &resources_json, &now)?;
        
        let mut final_worker = worker.clone();
        final_worker.status = WorkerStatus::Online;
        final_worker.last_heartbeat = now;

        Ok(final_worker)
    }

    pub fn get_worker(&self, id: &str) -> Result<Option<Worker>, AresError> {
        self.repo.get(id)
    }

    pub fn list_workers(&self) -> Result<Vec<Worker>, AresError> {
        self.repo.list()
    }

    pub fn delete_worker(&self, id: &str) -> Result<(), AresError> {
        self.repo.delete(id)
    }
}
