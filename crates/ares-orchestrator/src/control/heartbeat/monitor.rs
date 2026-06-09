use crate::control::workers::models::WorkerStatus;
use crate::control::workers::repository::WorkerRepository;
use crate::control::config::OrchestratorConfig;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::time::interval;
use tracing::{error, info, warn};

pub struct HeartbeatMonitor {
    repo: Arc<WorkerRepository>,
    config: OrchestratorConfig,
}

impl HeartbeatMonitor {
    pub fn new(repo: Arc<WorkerRepository>, config: OrchestratorConfig) -> Self {
        Self { repo, config }
    }

    pub fn start(self) {
        let repo = self.repo.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            info!("Starting HeartbeatMonitor background task");
            let mut ticker = interval(config.heartbeat_check_interval);

            loop {
                ticker.tick().await;

                if let Err(e) = Self::check_heartbeats(&repo, &config) {
                    error!("Error checking worker heartbeats: {}", e);
                }
            }
        });
    }

    fn check_heartbeats(repo: &WorkerRepository, config: &OrchestratorConfig) -> Result<(), ares_core::AresError> {
        let workers = repo.list()?;
        let now = Utc::now();

        for worker in workers {
            // Ignore already dead or offline workers to avoid unnecessary DB updates
            if worker.status == WorkerStatus::Dead || worker.status == WorkerStatus::Offline {
                continue;
            }

            if let Ok(last_hb) = DateTime::parse_from_rfc3339(&worker.last_heartbeat) {
                let duration_since_last_hb = now.signed_duration_since(last_hb.with_timezone(&Utc));

                if duration_since_last_hb.to_std().unwrap_or_default() > config.heartbeat_timeout {
                    warn!("Worker {} missed heartbeat. Marking as Offline.", worker.id);
                    
                    let resources_json = serde_json::to_string(&worker.resources).unwrap_or_default();
                    repo.update_status(
                        &worker.id,
                        &WorkerStatus::Offline,
                        &resources_json,
                        &worker.last_heartbeat,
                    )?;
                }
            }
        }

        Ok(())
    }
}
