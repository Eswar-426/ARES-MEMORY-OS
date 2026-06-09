mod common;

use ares_orchestrator::runtime::leases::repository::LeaseRepository;
use ares_orchestrator::runtime::leases::service::LeaseService;
use chrono::{Duration, Utc};
use std::sync::Arc;

#[test]
fn test_acquire_and_renew_lease() {
    let (store, config, _dir) = common::setup_test_env();
    let repo = Arc::new(LeaseRepository::new(store));
    let service = LeaseService::new(repo.clone(), config);

    let worker_id = "w1";
    let queue_id = "q1";
    let workflow_id = "wf1";
    let execution_id = "ex1";

    let lease = service.acquire_lease(worker_id, queue_id, workflow_id, execution_id).expect("Should acquire lease");
    
    assert_eq!(lease.worker_id, worker_id);
    assert_eq!(lease.queue_id, queue_id);

    // Renew lease
    let renew_result = service.renew_lease(&lease.id);
    assert!(renew_result.is_ok(), "Should renew lease");
}

#[test]
fn test_double_acquire_rejected() {
    let (store, config, _dir) = common::setup_test_env();
    let repo = Arc::new(LeaseRepository::new(store));
    let service = LeaseService::new(repo.clone(), config);

    let worker_id = "w1";
    let queue_id = "q1";
    let workflow_id = "wf1";
    let execution_id = "ex1";

    let _lease = service.acquire_lease(worker_id, queue_id, workflow_id, execution_id).unwrap();

    // Trying to insert the identical queue_id again directly should fail via Unique Constraint
    // Wait, the acquire_lease creates a new UUID for lease.id. The constraint in sqlite for job_leases:
    // `execution_id TEXT UNIQUE NOT NULL` or `queue_id TEXT UNIQUE NOT NULL`?
    // Let's attempt to acquire a lease for the same execution_id
    let res = service.acquire_lease(worker_id, queue_id, workflow_id, execution_id);
    assert!(res.is_err(), "Should reject double acquire on same execution/queue");
}

#[test]
fn test_expire_and_recover_lease() {
    let (store, config, _dir) = common::setup_test_env();
    let repo = Arc::new(LeaseRepository::new(store));
    let service = LeaseService::new(repo.clone(), config);

    let worker_id = "w1";
    let queue_id = "q1";
    let workflow_id = "wf1";
    let execution_id = "ex1";

    let lease = service.acquire_lease(worker_id, queue_id, workflow_id, execution_id).unwrap();

    // Manually force expiration by updating the DB via repo (simulate time passing)
    let past = Utc::now() - Duration::seconds(100);
    repo.renew(&lease.id, &past.to_rfc3339()).unwrap();

    // Now find expired
    let expired = repo.find_expired().unwrap();
    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0].id, lease.id);

    // Delete it (simulate recovery)
    service.release_lease(&lease.id).unwrap();
    
    let expired_after = repo.find_expired().unwrap();
    assert_eq!(expired_after.len(), 0);
}
