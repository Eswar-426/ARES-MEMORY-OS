use super::{dto::*, models::*};
use chrono::Utc;
use uuid::Uuid;

pub struct WorkerMapper;

impl WorkerMapper {
    pub fn from_registration(req: WorkerRegistrationRequest) -> Worker {
        let now = Utc::now().to_rfc3339();
        Worker {
            id: Uuid::now_v7().to_string(),
            hostname: req.hostname,
            capabilities: req.capabilities,
            labels: req.labels,
            status: WorkerStatus::Registering,
            resources: req.resources,
            registered_at: now.clone(),
            last_heartbeat: now,
        }
    }
}
