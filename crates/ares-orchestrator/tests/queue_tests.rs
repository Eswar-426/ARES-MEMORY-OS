mod common;

use ares_orchestrator::runtime::queue::dto::EnqueueRequest;
use ares_orchestrator::runtime::queue::models::QueueStatus;
use ares_orchestrator::runtime::queue::repository::QueueRepository;
use ares_orchestrator::runtime::queue::service::QueueService;

#[test]
fn test_enqueue_and_dequeue() {
    let (store, _config, _dir) = common::setup_test_env();
    let repo = QueueRepository::new(store.clone());
    let service = QueueService::new(repo);

    let req = EnqueueRequest {
        workflow_id: "wf1".into(),
        priority: 10,
        execution_key: "exec_key_1".into(),
        execution_checksum: "chk1".into(),
    };

    let item = service.enqueue(req).expect("Should enqueue item");
    assert_eq!(item.status, QueueStatus::Queued);
    assert_eq!(item.workflow_id, "wf1");

    // Try dequeue using a fresh repo instance
    let repo2 = QueueRepository::new(store.clone());
    let unassigned = repo2.dequeue_unassigned(10).expect("Should dequeue");
    assert_eq!(unassigned.len(), 1);
    assert_eq!(unassigned[0].id, item.id);
}

#[test]
fn test_duplicate_execution_key_rejected() {
    let (store, _config, _dir) = common::setup_test_env();
    let repo = QueueRepository::new(store.clone());
    let service = QueueService::new(repo);

    let req1 = EnqueueRequest {
        workflow_id: "wf1".into(),
        priority: 10,
        execution_key: "duplicate_key".into(),
        execution_checksum: "chk1".into(),
    };

    service.enqueue(req1).expect("First enqueue should succeed");

    let req2 = EnqueueRequest {
        workflow_id: "wf2".into(),
        priority: 5,
        execution_key: "duplicate_key".into(),
        execution_checksum: "chk2".into(),
    };

    let res = service.enqueue(req2);
    assert!(res.is_err(), "Duplicate execution key must be rejected");
    
    // Check error string contains conflict or unique
    let err_str = res.unwrap_err().to_string();
    assert!(err_str.contains("Duplicate execution key") || err_str.contains("Conflict"));
}

#[test]
fn test_assign_worker_updates_status() {
    let (store, _config, _dir) = common::setup_test_env();
    let repo = QueueRepository::new(store.clone());
    let service = QueueService::new(repo);

    let req = EnqueueRequest {
        workflow_id: "wf1".into(),
        priority: 10,
        execution_key: "exec_key_2".into(),
        execution_checksum: "chk2".into(),
    };

    let item = service.enqueue(req).unwrap();

    // Assign worker
    service.assign_worker(&item.id, "worker_1").unwrap();

    // Verify it's no longer in unassigned
    let repo2 = QueueRepository::new(store.clone());
    let unassigned = repo2.dequeue_unassigned(10).unwrap();
    assert_eq!(unassigned.len(), 0);
    
    // Verify status update in DB via a raw query just to be sure, or rely on dequeue logic
}

#[test]
fn test_complete_fail_retry_logic() {
    let (store, _config, _dir) = common::setup_test_env();
    let repo = QueueRepository::new(store.clone());
    let service = QueueService::new(repo);

    let req = EnqueueRequest {
        workflow_id: "wf_test_status".into(),
        priority: 10,
        execution_key: "exec_key_3".into(),
        execution_checksum: "chk3".into(),
    };

    let item = service.enqueue(req).unwrap();

    let repo2 = QueueRepository::new(store.clone());

    // Test Complete
    repo2.update_status(&item.id, &QueueStatus::Completed, None, None, Some("2026-01-01T00:00:00Z")).unwrap();

    // Re-enqueue another
    let req2 = EnqueueRequest {
        workflow_id: "wf_test_status2".into(),
        priority: 10,
        execution_key: "exec_key_4".into(),
        execution_checksum: "chk4".into(),
    };
    let item2 = service.enqueue(req2).unwrap();

    // Test Fail
    repo2.update_status(&item2.id, &QueueStatus::Failed, None, None, None).unwrap();
    
    // Re-enqueue another
    let req3 = EnqueueRequest {
        workflow_id: "wf_test_status3".into(),
        priority: 10,
        execution_key: "exec_key_5".into(),
        execution_checksum: "chk5".into(),
    };
    let item3 = service.enqueue(req3).unwrap();
    repo2.update_status(&item3.id, &QueueStatus::Retrying, None, None, None).unwrap();
}
