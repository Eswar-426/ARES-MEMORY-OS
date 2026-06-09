use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub enum WorkerStatus {
    Registering,
    Online,
    Busy,
    Draining,
    Offline,
    Dead,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct WorkerCapability {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct WorkerResources {
    pub cpu: f32,    // Cores
    pub memory: u64, // MB
    pub disk: u64,   // MB
    pub available_cpu: f32,
    pub available_memory: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Worker {
    pub id: String,
    pub hostname: String,
    pub capabilities: Vec<WorkerCapability>,
    pub labels: HashMap<String, String>,
    pub status: WorkerStatus,
    pub resources: WorkerResources,
    pub registered_at: String,
    pub last_heartbeat: String,
}
