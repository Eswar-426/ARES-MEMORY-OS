use super::repository::LeaseRepository;
use crate::control::config::OrchestratorConfig;
use crate::runtime::queue::models::QueueStatus;
use crate::runtime::queue::repository::QueueRepository;
use std::sync::Arc;
use tokio::time::interval;
use tracing::{error, info, warn};

pub struct LeaseRecoveryTask {
    lease_repo: Arc<LeaseRepository>,
    queue_repo: Arc<QueueRepository>,
    config: OrchestratorConfig,
}

impl LeaseRecoveryTask {
    pub fn new(
        lease_repo: Arc<LeaseRepository>,
        queue_repo: Arc<QueueRepository>,
        config: OrchestratorConfig,
    ) -> Self {
        Self {
            lease_repo,
            queue_repo,
            config,
        }
    }

    pub fn start(self) {
        let lease_repo = self.lease_repo.clone();
        let queue_repo = self.queue_repo.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            info!("Starting LeaseRecoveryTask background worker");
            let mut ticker = interval(config.lease_renewal_interval);

            loop {
                ticker.tick().await;

                if let Err(e) = Self::recover_expired_leases(&lease_repo, &queue_repo) {
                    error!("Error recovering expired leases: {}", e);
                }
            }
        });
    }

    fn recover_expired_leases(
        lease_repo: &LeaseRepository,
        queue_repo: &QueueRepository,
    ) -> Result<(), ares_core::AresError> {
        let expired_leases = lease_repo.find_expired()?;

        for lease in expired_leases {
            warn!(
                "Lease {} expired for worker {}. Recovering queue item {}",
                lease.id, lease.worker_id, lease.queue_id
            );

            // Re-queue the job (Orphaned or Queued depending on retry logic)
            // For now, let's mark it back to Queued and clear the assigned worker
            queue_repo.update_status(&lease.queue_id, &QueueStatus::Orphaned, None, None, None)?;

            // Delete the expired lease
            lease_repo.delete(&lease.id)?;
        }

        Ok(())
    }
}
