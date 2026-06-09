use crate::control::workers::models::WorkerStatus;
use crate::control::workers::repository::WorkerRepository;
use ares_core::AresError;
use chrono::Utc;

pub struct HeartbeatService {
    repo: WorkerRepository,
}

impl HeartbeatService {
    pub fn new(repo: WorkerRepository) -> Self {
        Self { repo }
    }

    pub fn process_heartbeat(&self, id: &str, available_cpu: f32, available_memory: u64) -> Result<(), AresError> {
        let worker = self.repo.get(id)?;
        if let Some(mut worker) = worker {
            // Update resources
            worker.resources.available_cpu = available_cpu;
            worker.resources.available_memory = available_memory;
            
            let resources_json = serde_json::to_string(&worker.resources).unwrap_or_default();
            let now = Utc::now().to_rfc3339();
            
            // If it was Offline/Dead, mark it Online since it's heartbeating again
            let next_status = match worker.status {
                WorkerStatus::Offline | WorkerStatus::Dead | WorkerStatus::Registering => WorkerStatus::Online,
                other => other,
            };

            self.repo.update_status(id, &next_status, &resources_json, &now)?;
        } else {
            return Err(AresError::validation("Worker not found for heartbeat"));
        }
        Ok(())
    }
}
