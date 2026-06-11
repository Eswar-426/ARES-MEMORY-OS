use ares_intelligence::learning::profiler::ModelProfiler;
use ares_intelligence::learning::service::LearningService;
use ares_intelligence::models::capability::TaskType;
use uuid::Uuid;

#[test]
fn test_success_rate_ema_decay() {
    let profiler = ModelProfiler::new(0.5, 0.2); // Alpha 0.5 for fast response
    let service = LearningService::new(profiler);
    let model_id = Uuid::now_v7();

    // First run (fail): initial success_rate is 1.0.
    // new_rate = (0.5 * 0.0) + (0.5 * 1.0) = 0.5
    service.process_execution_result(model_id, TaskType::Reasoning, false, 500);

    let profile = service.get_profile(model_id).unwrap();
    assert_eq!(profile.success_rate, 0.5);
    assert_eq!(profile.total_executions, 1);

    // Second run (fail):
    // new_rate = (0.5 * 0.0) + (0.5 * 0.5) = 0.25
    service.process_execution_result(model_id, TaskType::Reasoning, false, 500);

    let profile = service.get_profile(model_id).unwrap();
    assert_eq!(profile.success_rate, 0.25);
    assert_eq!(profile.total_executions, 2);
}

#[test]
fn test_success_rate_ema_recovery() {
    let profiler = ModelProfiler::new(0.5, 0.2);
    let service = LearningService::new(profiler);
    let model_id = Uuid::now_v7();

    // Force a few failures to drop rate
    service.process_execution_result(model_id, TaskType::Reasoning, false, 500); // 0.5
    service.process_execution_result(model_id, TaskType::Reasoning, false, 500); // 0.25

    // Now succeed:
    // new_rate = (0.5 * 1.0) + (0.5 * 0.25) = 0.5 + 0.125 = 0.625
    service.process_execution_result(model_id, TaskType::Reasoning, true, 500);

    let profile = service.get_profile(model_id).unwrap();
    assert_eq!(profile.success_rate, 0.625);
}

#[test]
fn test_success_rate_bounds() {
    let profiler = ModelProfiler::new(1.0, 0.2); // Instant override
    let service = LearningService::new(profiler);
    let model_id = Uuid::now_v7();

    // Fail immediately -> drops to 0.0
    service.process_execution_result(model_id, TaskType::Reasoning, false, 500);

    let profile = service.get_profile(model_id).unwrap();
    assert_eq!(profile.success_rate, 0.0);

    // It shouldn't drop below 0.0 even with another failure (clamped)
    service.process_execution_result(model_id, TaskType::Reasoning, false, 500);
    let profile2 = service.get_profile(model_id).unwrap();
    assert_eq!(profile2.success_rate, 0.0);
}

#[test]
fn test_latency_ema_adaptation() {
    let profiler = ModelProfiler::new(0.1, 0.2); // Alpha 0.2 for latency
    let service = LearningService::new(profiler);
    let model_id = Uuid::now_v7();

    // Initial default latency is 1000.
    // Run at 200ms:
    // new_lat = (0.2 * 200) + (0.8 * 1000) = 40 + 800 = 840
    service.process_execution_result(model_id, TaskType::Reasoning, true, 200);

    let profile = service.get_profile(model_id).unwrap();
    assert_eq!(profile.average_latency_ms, 840);

    // Run at 200ms again:
    // new_lat = (0.2 * 200) + (0.8 * 840) = 40 + 672 = 712
    service.process_execution_result(model_id, TaskType::Reasoning, true, 200);

    let profile2 = service.get_profile(model_id).unwrap();
    assert_eq!(profile2.average_latency_ms, 712);
}

#[test]
fn test_latency_minimum_bound() {
    let profiler = ModelProfiler::new(0.1, 1.0); // Instant update
    let service = LearningService::new(profiler);
    let model_id = Uuid::now_v7();

    // Report 0ms latency, it should clamp to at least 1 to avoid div by zero in scoring
    service.process_execution_result(model_id, TaskType::Reasoning, true, 0);

    let profile = service.get_profile(model_id).unwrap();
    assert_eq!(profile.average_latency_ms, 1);
}

#[test]
fn test_independent_model_profiles() {
    let service = LearningService::default();
    let m1 = Uuid::now_v7();
    let m2 = Uuid::now_v7();

    // M1 fails
    service.process_execution_result(m1, TaskType::Reasoning, false, 1000);
    // M2 succeeds and is very fast
    service.process_execution_result(m2, TaskType::Reasoning, true, 100);

    let p1 = service.get_profile(m1).unwrap();
    let p2 = service.get_profile(m2).unwrap();

    assert!(p1.success_rate < p2.success_rate);
    assert!(p1.average_latency_ms > p2.average_latency_ms);
}
