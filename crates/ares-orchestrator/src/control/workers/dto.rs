use super::models::{WorkerCapability, WorkerResources};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct WorkerRegistrationRequest {
    pub hostname: String,
    pub capabilities: Vec<WorkerCapability>,
    pub labels: HashMap<String, String>,
    pub resources: WorkerResources,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct WorkerStatusUpdateRequest {
    pub available_cpu: f32,
    pub available_memory: u64,
}
