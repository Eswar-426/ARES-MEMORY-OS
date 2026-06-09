use super::{models::JobLease, repository::LeaseRepository};
use crate::control::config::OrchestratorConfig;
use ares_core::AresError;
use chrono::{Duration, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct LeaseService {
    repo: Arc<LeaseRepository>,
    config: OrchestratorConfig,
}

impl LeaseService {
    pub fn new(repo: Arc<LeaseRepository>, config: OrchestratorConfig) -> Self {
        Self { repo, config }
    }

    pub fn acquire_lease(
        &self,
        worker_id: &str,
        queue_id: &str,
        workflow_id: &str,
        execution_id: &str,
    ) -> Result<JobLease, AresError> {
        let now = Utc::now();
        let expires_at = now
            + chrono::Duration::from_std(self.config.default_lease_duration)
                .unwrap_or(Duration::seconds(60));

        let lease = JobLease {
            id: Uuid::now_v7().to_string(),
            worker_id: worker_id.to_string(),
            queue_id: queue_id.to_string(),
            workflow_id: workflow_id.to_string(),
            execution_id: execution_id.to_string(),
            acquired_at: now.to_rfc3339(),
            expires_at: expires_at.to_rfc3339(),
        };

        self.repo.acquire(&lease)?;
        Ok(lease)
    }

    pub fn renew_lease(&self, lease_id: &str) -> Result<(), AresError> {
        let expires_at = Utc::now()
            + chrono::Duration::from_std(self.config.default_lease_duration)
                .unwrap_or(Duration::seconds(60));
        self.repo.renew(lease_id, &expires_at.to_rfc3339())
    }

    pub fn release_lease(&self, lease_id: &str) -> Result<(), AresError> {
        self.repo.delete(lease_id)
    }
}
