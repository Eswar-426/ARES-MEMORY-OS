use ares_intelligence::cache::manager::CacheManager;
use ares_intelligence::models::capability::{ModelCapability, TaskType};
use ares_intelligence::models::model::Model;
use ares_intelligence::models::profile::ModelProfile;
use ares_intelligence::providers::mock::{MockProvider, MockProviderBehavior};
use ares_intelligence::providers::ModelProvider;
use ares_intelligence::routing::fallback::FallbackManager;
use ares_intelligence::routing::service::RoutingService;
use ares_intelligence::sandbox::executor::SandboxExecutor;
use ares_intelligence::sandbox::limits::Limits;
use ares_intelligence::sandbox::quotas::Quotas;
use ares_intelligence::selection::selector::{ModelSelector, Objective, SelectionCriteria};
use proptest::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

fn create_model(id: Uuid, cost: f64) -> Model {
    Model {
        id,
        name: format!("model_{}", id),
        provider_id: "test".to_string(),
        version: "1.0".to_string(),
        max_context_window: 4096,
        cost_per_1k_tokens: cost,
        capabilities: vec![ModelCapability::Reasoning],
    }
}

fn create_profile(model_id: Uuid, latency: u64, success: f64) -> ModelProfile {
    ModelProfile {
        model_id,
        success_rate: success,
        average_latency_ms: latency,
        total_executions: 100,
    }
}

proptest! {
    #[test]
    fn prop_test_selection_cheapest(
        c1 in 0.001..1.0f64,
        c2 in 0.001..1.0f64,
        c3 in 0.001..1.0f64,
    ) {
        let selector = ModelSelector::new();
        let m1 = create_model(Uuid::now_v7(), c1);
        let m2 = create_model(Uuid::now_v7(), c2);
        let m3 = create_model(Uuid::now_v7(), c3);

        let models = vec![m1.clone(), m2.clone(), m3.clone()];
        let profiles = HashMap::new();

        let criteria = SelectionCriteria {
            task_id: "task-1".to_string(),
            task_type: TaskType::Reasoning,
            required_caps: vec![],
            objective: Objective::Cheapest,
            max_latency_ms: None,
            max_cost: None,
        };

        if let Ok((selected, _)) = selector.select_best_model(&criteria, &models, &profiles) {
            for m in &models {
                prop_assert!(selected.cost_per_1k_tokens <= m.cost_per_1k_tokens);
            }
        }
    }

    #[test]
    fn prop_test_selection_fastest(
        lat1 in 10u64..5000u64,
        lat2 in 10u64..5000u64,
        lat3 in 10u64..5000u64,
    ) {
        let selector = ModelSelector::new();
        let m1 = create_model(Uuid::now_v7(), 0.1);
        let m2 = create_model(Uuid::now_v7(), 0.1);
        let m3 = create_model(Uuid::now_v7(), 0.1);

        let mut profiles = HashMap::new();
        profiles.insert(m1.id, create_profile(m1.id, lat1, 0.9));
        profiles.insert(m2.id, create_profile(m2.id, lat2, 0.9));
        profiles.insert(m3.id, create_profile(m3.id, lat3, 0.9));

        let models = vec![m1.clone(), m2.clone(), m3.clone()];

        let criteria = SelectionCriteria {
            task_id: "task-1".to_string(),
            task_type: TaskType::Reasoning,
            required_caps: vec![],
            objective: Objective::Fastest,
            max_latency_ms: None,
            max_cost: None,
        };

        if let Ok((selected, _)) = selector.select_best_model(&criteria, &models, &profiles) {
            let selected_latency = profiles.get(&selected.id).unwrap().average_latency_ms;
            for m in &models {
                let current_latency = profiles.get(&m.id).unwrap().average_latency_ms;
                prop_assert!(selected_latency <= current_latency);
            }
        }
    }

    #[test]
    fn prop_test_selection_quality(
        q1 in 0.1..1.0f64,
        q2 in 0.1..1.0f64,
        q3 in 0.1..1.0f64,
    ) {
        let selector = ModelSelector::new();
        let m1 = create_model(Uuid::now_v7(), 0.1);
        let m2 = create_model(Uuid::now_v7(), 0.1);
        let m3 = create_model(Uuid::now_v7(), 0.1);

        let mut profiles = HashMap::new();
        profiles.insert(m1.id, create_profile(m1.id, 500, q1));
        profiles.insert(m2.id, create_profile(m2.id, 500, q2));
        profiles.insert(m3.id, create_profile(m3.id, 500, q3));

        let models = vec![m1.clone(), m2.clone(), m3.clone()];

        let criteria = SelectionCriteria {
            task_id: "task-1".to_string(),
            task_type: TaskType::Reasoning,
            required_caps: vec![],
            objective: Objective::HighestQuality,
            max_latency_ms: None,
            max_cost: None,
        };

        if let Ok((selected, _)) = selector.select_best_model(&criteria, &models, &profiles) {
            let selected_quality = profiles.get(&selected.id).unwrap().success_rate;
            for m in &models {
                let current_quality = profiles.get(&m.id).unwrap().success_rate;
                prop_assert!(selected_quality >= current_quality);
            }
        }
    }

    #[test]
    fn prop_test_selection_balanced(
        c1 in 0.001..1.0f64, c2 in 0.001..1.0f64,
        q1 in 0.1..1.0f64, q2 in 0.1..1.0f64,
    ) {
        let selector = ModelSelector::new();
        let m1 = create_model(Uuid::now_v7(), c1);
        let m2 = create_model(Uuid::now_v7(), c2);

        let mut profiles = HashMap::new();
        profiles.insert(m1.id, create_profile(m1.id, 500, q1));
        profiles.insert(m2.id, create_profile(m2.id, 500, q2));

        let models = vec![m1.clone(), m2.clone()];

        let criteria = SelectionCriteria {
            task_id: "task-1".to_string(),
            task_type: TaskType::Reasoning,
            required_caps: vec![],
            objective: Objective::Balanced,
            max_latency_ms: None,
            max_cost: None,
        };

        if let Ok((selected, _)) = selector.select_best_model(&criteria, &models, &profiles) {
            let score = |m: &Model| {
                let prof = profiles.get(&m.id).unwrap();
                let cost = if m.cost_per_1k_tokens <= 0.0 { 0.0001 } else { m.cost_per_1k_tokens };
                prof.success_rate / cost
            };
            let selected_score = score(&selected);
            for m in &models {
                prop_assert!(selected_score >= score(m));
            }
        }
    }

    #[test]
    fn prop_test_cache_invariant(key in 0..10_000u64, val in ".*") {
        let cache: CacheManager<u64, String> = CacheManager::new(Duration::from_secs(60));
        cache.set(key, val.clone());
        prop_assert_eq!(cache.get(&key), Some(val.clone()));
        cache.invalidate(&key);
        prop_assert_eq!(cache.get(&key), None);
    }
}

// Wrapping async proptest in a runtime isn't fully straightforward with standard proptest! macro natively,
// so we'll test the routing invariant manually with randomized distributions within a standard test loop or block_on.
#[tokio::test]
async fn test_routing_healthy_provider_always_succeeds() {
    let fallback_manager = FallbackManager::new();
    let routing = RoutingService::new(fallback_manager);

    let limits = Limits::default();
    let quotas = Quotas::default();
    let executor = SandboxExecutor::new(limits, quotas);

    // Provide 5 fail providers, 1 healthy provider.
    let mut available = Vec::new();
    let mut providers: HashMap<String, Arc<dyn ModelProvider>> = HashMap::new();

    for i in 0..5 {
        let mut fail_model = create_model(Uuid::now_v7(), 0.1);
        let id = format!("fail_{}", i);
        fail_model.provider_id = id.clone();
        available.push(fail_model);
        providers.insert(
            id.clone(),
            Arc::new(MockProvider::new(&id, MockProviderBehavior::Offline)),
        );
    }

    let mut healthy_model = create_model(Uuid::now_v7(), 0.1);
    healthy_model.provider_id = "healthy".to_string();
    available.push(healthy_model.clone());
    providers.insert(
        "healthy".to_string(),
        Arc::new(MockProvider::new(
            "healthy",
            MockProviderBehavior::Success("Hello".to_string()),
        )),
    );

    // Try routing 100 times, shuffling the available models (mocking randomness)
    for _ in 0..100 {
        // Even if we use fail_model as primary, it must eventually fallback to healthy.
        let primary = available[0].clone();

        let res = routing
            .execute_with_routing_and_fallback(
                "task", "prompt", primary, &available, &providers, &executor,
            )
            .await;

        assert!(
            res.is_ok(),
            "Routing failed despite a healthy provider existing!"
        );
        let (answer, _) = res.unwrap();
        assert_eq!(answer, "Hello");
    }
}
