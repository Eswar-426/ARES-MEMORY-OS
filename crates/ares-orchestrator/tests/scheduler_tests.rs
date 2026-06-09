use ares_orchestrator::control::scheduler::strategy::{
    CapabilityAwareStrategy, LeastLoadedStrategy, PriorityStrategy, RoundRobinStrategy, SchedulingStrategy,
};
use ares_orchestrator::control::workers::models::{Worker, WorkerCapability, WorkerResources, WorkerStatus};
use std::collections::HashMap;

fn mock_worker(id: &str, status: WorkerStatus, memory: u64, labels: Vec<(&str, &str)>, caps: Vec<(&str, &str)>) -> Worker {
    let mut label_map = HashMap::new();
    for (k, v) in labels {
        label_map.insert(k.to_string(), v.to_string());
    }

    let capabilities = caps
        .into_iter()
        .map(|(n, v)| WorkerCapability {
            name: n.to_string(),
            version: v.to_string(),
        })
        .collect();

    Worker {
        id: id.to_string(),
        hostname: format!("{}-host", id),
        capabilities,
        labels: label_map,
        status,
        resources: WorkerResources {
            cpu: 4.0,
            memory: 16000,
            disk: 100000,
            available_cpu: 4.0,
            available_memory: memory,
        },
        registered_at: "0".to_string(),
        last_heartbeat: "0".to_string(),
    }
}

#[test]
fn test_least_loaded_strategy() {
    let strategy = LeastLoadedStrategy;

    let workers = vec![
        mock_worker("w1", WorkerStatus::Online, 1000, vec![], vec![]),
        mock_worker("w2", WorkerStatus::Online, 5000, vec![], vec![]), // Most free memory
        mock_worker("w3", WorkerStatus::Online, 2000, vec![], vec![]),
    ];

    let selected = strategy.select_worker(&workers, &[]).expect("Should select a worker");
    assert_eq!(selected.id, "w2");
}

#[test]
fn test_least_loaded_ignores_offline_and_dead() {
    let strategy = LeastLoadedStrategy;

    let workers = vec![
        mock_worker("w1", WorkerStatus::Offline, 10000, vec![], vec![]),
        mock_worker("w2", WorkerStatus::Dead, 20000, vec![], vec![]),
        mock_worker("w3", WorkerStatus::Online, 2000, vec![], vec![]),
    ];

    let selected = strategy.select_worker(&workers, &[]).expect("Should select a worker");
    assert_eq!(selected.id, "w3"); // Only w3 is online
}

#[test]
fn test_capability_aware_strategy_exact_match() {
    let strategy = CapabilityAwareStrategy;

    let workers = vec![
        mock_worker("w1", WorkerStatus::Online, 1000, vec![], vec![("rust", "1.0")]),
        mock_worker("w2", WorkerStatus::Online, 1000, vec![], vec![("python", "3.9"), ("rust", "1.0")]),
    ];

    let req = vec![
        WorkerCapability { name: "python".into(), version: "3.9".into() },
        WorkerCapability { name: "rust".into(), version: "1.0".into() },
    ];

    let selected = strategy.select_worker(&workers, &req).expect("Should select w2");
    assert_eq!(selected.id, "w2");
}

#[test]
fn test_capability_aware_strategy_version_mismatch() {
    let strategy = CapabilityAwareStrategy;

    let workers = vec![
        mock_worker("w1", WorkerStatus::Online, 1000, vec![], vec![("python", "3.8")]),
    ];

    let req = vec![
        WorkerCapability { name: "python".into(), version: "3.9".into() },
    ];

    let selected = strategy.select_worker(&workers, &req);
    assert!(selected.is_none(), "Should return None on version mismatch");
}

#[test]
fn test_priority_strategy_ordering() {
    let strategy = PriorityStrategy;

    // PriorityStrategy currently uses simple alphabetical comparison of tier values
    // "medium" > "low" > "high" (m > l > h) => "medium" should win... wait, m is 109, l is 108.
    // Let's use numeric string values or clear alphabet
    
    let workers = vec![
        mock_worker("w1", WorkerStatus::Online, 1000, vec![("tier", "1-low")], vec![]),
        mock_worker("w2", WorkerStatus::Online, 1000, vec![("tier", "3-high")], vec![]),
        mock_worker("w3", WorkerStatus::Online, 1000, vec![("tier", "2-medium")], vec![]),
    ];

    let selected = strategy.select_worker(&workers, &[]).unwrap();
    assert_eq!(selected.id, "w2");
}

#[test]
fn test_round_robin_strategy() {
    let strategy = RoundRobinStrategy {};

    let workers = vec![
        mock_worker("w1", WorkerStatus::Online, 1000, vec![], vec![]),
    ];

    let selected = strategy.select_worker(&workers, &[]).unwrap();
    assert_eq!(selected.id, "w1");
}

#[test]
fn test_all_strategies_ignore_offline() {
    let least = LeastLoadedStrategy;
    let rr = RoundRobinStrategy {};
    let cap = CapabilityAwareStrategy;
    let prio = PriorityStrategy;

    let workers = vec![
        mock_worker("w1", WorkerStatus::Offline, 10000, vec![("tier", "9-high")], vec![("python", "3.9")]),
    ];

    let req = vec![WorkerCapability { name: "python".into(), version: "3.9".into() }];

    assert!(least.select_worker(&workers, &[]).is_none());
    assert!(rr.select_worker(&workers, &[]).is_none());
    assert!(cap.select_worker(&workers, &req).is_none());
    assert!(prio.select_worker(&workers, &[]).is_none());
}
