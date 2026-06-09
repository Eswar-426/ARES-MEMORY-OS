use super::repository::OutboxRepository;
use crate::control::config::OrchestratorConfig;
use crate::events::publisher::EventPublisher;
use std::sync::Arc;
use tokio::time::interval;
use tracing::{error, info, warn};

pub struct OutboxPublisherWorker {
    repo: Arc<OutboxRepository>,
    publisher: Arc<dyn EventPublisher>,
    config: OrchestratorConfig,
}

impl OutboxPublisherWorker {
    pub fn new(
        repo: Arc<OutboxRepository>,
        publisher: Arc<dyn EventPublisher>,
        config: OrchestratorConfig,
    ) -> Self {
        Self {
            repo,
            publisher,
            config,
        }
    }

    pub fn start(self) {
        let repo = self.repo.clone();
        let publisher = self.publisher.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            info!("Starting OutboxPublisherWorker background task");
            let mut ticker = interval(config.outbox_poll_interval);

            loop {
                ticker.tick().await;

                if let Err(e) = Self::process_outbox(&repo, &*publisher).await {
                    error!("Error processing outbox events: {}", e);
                }
            }
        });
    }

    pub async fn process_outbox(
        repo: &OutboxRepository,
        publisher: &dyn EventPublisher,
    ) -> Result<(), ares_core::AresError> {
        let pending_events = repo.fetch_pending(50)?;

        for event in pending_events {
            // Attempt to publish
            match publisher.publish(&event.topic, &event.payload).await {
                Ok(_) => {
                    repo.mark_published(&event.id)?;
                }
                Err(e) => {
                    warn!("Failed to publish outbox event {}: {}", event.id, e);
                    repo.increment_retry(&event.id)?;

                    if event.retry_count >= 5 {
                        error!(
                            "Outbox event {} reached max retries, marking failed",
                            event.id
                        );
                        repo.mark_failed(&event.id)?;
                    }
                }
            }
        }

        Ok(())
    }
}
