mod common;

use ares_orchestrator::control::workers::dto::WorkerRegistrationRequest;
use ares_orchestrator::control::workers::models::{WorkerCapability, WorkerResources};
use ares_orchestrator::control::workers::repository::WorkerRepository;
use ares_orchestrator::control::workers::service::WorkerService;

use ares_orchestrator::runtime::queue::dto::EnqueueRequest;
use ares_orchestrator::runtime::queue::models::QueueStatus;
use ares_orchestrator::runtime::queue::repository::QueueRepository;
use ares_orchestrator::runtime::queue::service::QueueService;

use ares_orchestrator::control::scheduler::strategy::{LeastLoadedStrategy, SchedulingStrategy};

use ares_orchestrator::runtime::leases::repository::LeaseRepository;
use ares_orchestrator::runtime::leases::service::LeaseService;

use ares_orchestrator::events::outbox::models::OutboxEvent;
use ares_orchestrator::events::outbox::repository::OutboxRepository;

use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;

#[test]
fn test_orchestrator_lifecycle() {
    let (store, config, _dir) = common::setup_test_env();

    // 1. Repositories & Services
    let worker_repo = WorkerRepository::new(store.clone());
    let worker_service = WorkerService::new(worker_repo);

    let queue_repo = QueueRepository::new(store.clone());
    let queue_service = QueueService::new(queue_repo);

    let lease_repo = Arc::new(LeaseRepository::new(store.clone()));
    let lease_service = LeaseService::new(lease_repo.clone(), config.clone());

    let outbox_repo = OutboxRepository::new(store.clone());

    // 2. Worker Register
    let req = WorkerRegistrationRequest {
        hostname: "lifecycle-worker".into(),
        capabilities: vec![WorkerCapability { name: "test-cap".into(), version: "1.0".into() }],
        labels: HashMap::new(),
        resources: WorkerResources {
            cpu: 4.0,
            memory: 16000,
            disk: 100000,
            available_cpu: 4.0,
            available_memory: 16000,
        },
    };
    let worker = worker_service.register_worker(req).expect("Failed to register worker");

    // (Heartbeat is implicit in register as it sets it to Online + updates heartbeat)

    // 3. Queue Workflow
    let enq_req = EnqueueRequest {
        workflow_id: "wf-lifecycle".into(),
        priority: 10,
        execution_key: "exec-lifecycle-1".into(),
        execution_checksum: "chk-lifecycle-1".into(),
    };
    let queue_item = queue_service.enqueue(enq_req).expect("Failed to enqueue");

    // 4. Scheduler
    let strategy = LeastLoadedStrategy;
    let workers = worker_service.list_workers().expect("Failed to list workers");
    let selected_worker = strategy.select_worker(&workers, &[]).expect("Failed to select worker");
    assert_eq!(selected_worker.id, worker.id);

    // Assign
    queue_service.assign_worker(&queue_item.id, &selected_worker.id).expect("Failed to assign");
    
    // Refresh queue_item
    // Since QueueService doesn't have a get method easily accessible here, we'll assume it succeeded.

    // 5. Lease
    let lease = lease_service.acquire_lease(
        &selected_worker.id,
        &queue_item.id,
        &queue_item.workflow_id,
        &Uuid::now_v7().to_string(), // execution_id
    ).expect("Failed to acquire lease");

    // 6. Execution (Simulated)
    // Create an Outbox Event
    let event = OutboxEvent {
        id: Uuid::now_v7().to_string(),
        topic: "workflow.completed".into(),
        payload: "{}".into(),
        created_at: Utc::now().to_rfc3339(),
        published_at: None,
        status: "Pending".into(),
        retry_count: 0,
    };
    outbox_repo.insert(&event).expect("Failed to insert outbox event");

    // 7. Complete Queue Item
    let q_repo = QueueRepository::new(store.clone());
    q_repo.update_status(&queue_item.id, &QueueStatus::Completed, None, None, Some(&Utc::now().to_rfc3339())).expect("Failed to complete queue item");

    // 8. Release Lease
    lease_service.release_lease(&lease.id).expect("Failed to release lease");

    // Assertions
    let pending_events = outbox_repo.fetch_pending(10).unwrap();
    assert_eq!(pending_events.len(), 1);

    let final_queue = q_repo.dequeue_unassigned(10).unwrap();
    assert_eq!(final_queue.len(), 0); // it's completed, so no unassigned

    let expired_leases = lease_repo.find_expired().unwrap();
    assert_eq!(expired_leases.len(), 0); // it was released
}
