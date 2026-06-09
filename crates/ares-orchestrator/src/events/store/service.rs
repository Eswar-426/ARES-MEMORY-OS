use super::repository::EventStoreRepository;
use crate::events::envelope::EventEnvelope;
use crate::events::outbox::models::OutboxEvent;
use crate::events::outbox::repository::OutboxRepository;
use ares_core::AresError;
use chrono::Utc;

pub struct EventStoreService {
    store_repo: EventStoreRepository,
    outbox_repo: OutboxRepository,
}

impl EventStoreService {
    pub fn new(store_repo: EventStoreRepository, outbox_repo: OutboxRepository) -> Self {
        Self {
            store_repo,
            outbox_repo,
        }
    }

    /// Appends an event to the Event Store and atomically publishes it to the Outbox.
    /// This enforces the strict architecture rule: Event Store -> Outbox -> Event Bus -> Consumers.
    pub fn append(&self, event: &EventEnvelope) -> Result<(), AresError> {
        // 1. Write to Event Store
        self.store_repo.insert(event)?;

        // 2. Write to Outbox
        let payload_str =
            serde_json::to_string(event).map_err(|e| AresError::Serialization(e.to_string()))?;
        let outbox_event = OutboxEvent {
            id: uuid::Uuid::new_v4().to_string(),
            topic: event.topic.clone(),
            payload: payload_str,
            created_at: Utc::now().to_rfc3339(),
            published_at: None,
            status: "Pending".to_string(),
            retry_count: 0,
        };
        self.outbox_repo.insert(&outbox_event)?;

        Ok(())
    }
}
