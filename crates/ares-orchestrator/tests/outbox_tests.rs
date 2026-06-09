mod common;

use ares_orchestrator::events::outbox::models::OutboxEvent;
use ares_orchestrator::events::outbox::repository::OutboxRepository;
use ares_orchestrator::events::outbox::worker::OutboxPublisherWorker;
use ares_orchestrator::events::publisher::EventPublisher;
use ares_core::AresError;
use std::sync::atomic::{AtomicUsize, Ordering};
use chrono::Utc;

struct MockPublisher {
    call_count: AtomicUsize,
    fail_first_n: usize,
}

#[async_trait::async_trait]
impl EventPublisher for MockPublisher {
    async fn publish(&self, _topic: &str, _payload: &str) -> Result<(), AresError> {
        let count = self.call_count.fetch_add(1, Ordering::SeqCst);
        if count < self.fail_first_n {
            Err(AresError::validation("Simulated failure"))
        } else {
            Ok(())
        }
    }
}

fn mock_event(id: &str) -> OutboxEvent {
    OutboxEvent {
        id: id.into(),
        topic: "test_topic".into(),
        payload: "{}".into(),
        created_at: Utc::now().to_rfc3339(),
        published_at: None,
        status: "Pending".into(),
        retry_count: 0,
    }
}

#[tokio::test]
async fn test_insert_and_publish() {
    let (store, _config, _dir) = common::setup_test_env();
    let repo = OutboxRepository::new(store);
    
    let event = mock_event("evt1");
    repo.insert(&event).unwrap();

    let pending = repo.fetch_pending(10).unwrap();
    assert_eq!(pending.len(), 1);

    let publisher = MockPublisher {
        call_count: AtomicUsize::new(0),
        fail_first_n: 0,
    };

    OutboxPublisherWorker::process_outbox(&repo, &publisher).await.unwrap();

    let pending_after = repo.fetch_pending(10).unwrap();
    assert_eq!(pending_after.len(), 0);
    assert_eq!(publisher.call_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_duplicate_event() {
    let (store, _config, _dir) = common::setup_test_env();
    let repo = OutboxRepository::new(store);
    
    let event = mock_event("evt2");
    repo.insert(&event).unwrap();
    
    // Insert identical ID should fail (PrimaryKey)
    let res = repo.insert(&event);
    assert!(res.is_err(), "Duplicate OutboxEvent should fail");
}

#[tokio::test]
async fn test_publisher_failure_and_recovery() {
    let (store, _config, _dir) = common::setup_test_env();
    let repo = OutboxRepository::new(store);
    
    let event = mock_event("evt3");
    repo.insert(&event).unwrap();

    let publisher = MockPublisher {
        call_count: AtomicUsize::new(0),
        fail_first_n: 2, // Fails first 2 times, then succeeds
    };

    // Attempt 1 (Fails)
    OutboxPublisherWorker::process_outbox(&repo, &publisher).await.unwrap();
    
    let pending = repo.fetch_pending(10).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].retry_count, 1);
    assert_eq!(pending[0].status, "Pending");

    // Attempt 2 (Fails)
    OutboxPublisherWorker::process_outbox(&repo, &publisher).await.unwrap();
    
    let pending2 = repo.fetch_pending(10).unwrap();
    assert_eq!(pending2[0].retry_count, 2);

    // Attempt 3 (Succeeds)
    OutboxPublisherWorker::process_outbox(&repo, &publisher).await.unwrap();
    
    let pending3 = repo.fetch_pending(10).unwrap();
    assert_eq!(pending3.len(), 0); // successfully published
}

#[tokio::test]
async fn test_publisher_max_retries_failed() {
    let (store, _config, _dir) = common::setup_test_env();
    let repo = OutboxRepository::new(store);
    
    let event = mock_event("evt4");
    repo.insert(&event).unwrap();

    let publisher = MockPublisher {
        call_count: AtomicUsize::new(0),
        fail_first_n: 10, // Fails forever
    };

    // Retry 6 times to reach the threshold (5)
    for _ in 0..6 {
        OutboxPublisherWorker::process_outbox(&repo, &publisher).await.unwrap();
    }

    // After 5 retries it should be marked Failed and not show up in pending
    let pending = repo.fetch_pending(10).unwrap();
    assert_eq!(pending.len(), 0);
}
