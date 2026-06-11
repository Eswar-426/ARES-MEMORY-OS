use ares_intelligence::cache::manager::CacheManager;
use ares_intelligence::learning::profiler::ModelProfiler;
use ares_intelligence::models::capability::TaskType;
use std::time::Duration;
use uuid::Uuid;

#[test]
fn test_stability_10000_profile_updates() {
    let profiler = ModelProfiler::new(0.1, 0.2);
    let model_id = Uuid::now_v7();

    for i in 0..10_000 {
        // Toggle success randomly or deterministically
        let success = i % 3 != 0;
        profiler.update_success_rate(model_id, TaskType::Reasoning, success);
        profiler.record_latency(model_id, (i % 1000) as u64);
    }

    let profile = profiler.get_profile(model_id).unwrap();
    // Verify it didn't panic, overflow, or NaN
    assert_eq!(profile.total_executions, 10_000);
    assert!(profile.success_rate >= 0.0 && profile.success_rate <= 1.0);
    assert!(profile.average_latency_ms > 0);
}

#[tokio::test]
async fn test_stability_10000_cache_writes() {
    let cache: CacheManager<u64, String> = CacheManager::new(Duration::from_millis(10));

    for i in 0..10_000 {
        cache.set(i, "data".to_string());

        // Intermittently clear to simulate GC/eviction so it doesn't grow bounds
        if i % 1000 == 0 {
            cache.invalidate_all();
        }
    }

    // Sleep to allow the final batch to expire
    tokio::time::sleep(Duration::from_millis(15)).await;

    // Accessing an expired element forces lazy eviction
    for i in 9000..10_000 {
        assert_eq!(cache.get(&i), None);
    }
}
