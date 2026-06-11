use ares_intelligence::collaboration::service::CollaborationService;
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
use std::collections::HashMap;
use std::sync::Arc;
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

#[test]
fn test_determinism_selection_and_explanation() {
    let selector = ModelSelector::new();
    let m1 = create_model(Uuid::now_v7(), 0.05);
    let m2 = create_model(Uuid::now_v7(), 0.05);
    let m3 = create_model(Uuid::now_v7(), 0.05);

    let mut profiles = HashMap::new();
    profiles.insert(m1.id, create_profile(m1.id, 500, 0.8));
    profiles.insert(m2.id, create_profile(m2.id, 200, 0.5));
    profiles.insert(m3.id, create_profile(m3.id, 800, 0.99)); // Highest Quality

    let models = vec![m1.clone(), m2.clone(), m3.clone()];

    let criteria = SelectionCriteria {
        task_id: "task-1".to_string(),
        task_type: TaskType::Reasoning,
        required_caps: vec![],
        objective: Objective::HighestQuality,
        max_latency_ms: None,
        max_cost: None,
    };

    let (baseline_model, baseline_exp) = selector
        .select_best_model(&criteria, &models, &profiles)
        .unwrap();

    // Verify exactly the same outcome over 100 iterations
    for _ in 0..100 {
        let (selected, exp) = selector
            .select_best_model(&criteria, &models, &profiles)
            .unwrap();

        assert_eq!(selected.id, baseline_model.id);
        assert_eq!(exp.selected_model_id, baseline_exp.selected_model_id);
        assert_eq!(exp.reasoning, baseline_exp.reasoning);
        assert_eq!(
            exp.rejected_models.len(),
            baseline_exp.rejected_models.len()
        );

        for (i, rejected) in exp.rejected_models.iter().enumerate() {
            assert_eq!(rejected.model_id, baseline_exp.rejected_models[i].model_id);
            assert_eq!(rejected.reason, baseline_exp.rejected_models[i].reason);
        }
    }
}

#[tokio::test]
async fn test_determinism_routing_and_fallback_ordering() {
    let fallback_manager = FallbackManager::new();
    let routing = RoutingService::new(fallback_manager);

    let limits = Limits::default();
    let quotas = Quotas::default();
    let executor = SandboxExecutor::new(limits, quotas);

    let mut m1 = create_model(Uuid::now_v7(), 0.1);
    m1.provider_id = "prov_fail1".to_string();
    let mut m2 = create_model(Uuid::now_v7(), 0.1);
    m2.provider_id = "prov_fail2".to_string();
    let mut m3 = create_model(Uuid::now_v7(), 0.1);
    m3.provider_id = "prov_success".to_string();

    let available = vec![m1.clone(), m2.clone(), m3.clone()];

    let mut providers: HashMap<String, Arc<dyn ModelProvider>> = HashMap::new();
    providers.insert(
        "prov_fail1".to_string(),
        Arc::new(MockProvider::new(
            "prov_fail1",
            MockProviderBehavior::Offline,
        )),
    );
    providers.insert(
        "prov_fail2".to_string(),
        Arc::new(MockProvider::new(
            "prov_fail2",
            MockProviderBehavior::RateLimit,
        )),
    );
    providers.insert(
        "prov_success".to_string(),
        Arc::new(MockProvider::new(
            "prov_success",
            MockProviderBehavior::Success("Fallback OK".to_string()),
        )),
    );

    let (baseline_res, baseline_exp) = routing
        .execute_with_routing_and_fallback(
            "task-1",
            "prompt",
            m1.clone(),
            &available,
            &providers,
            &executor,
        )
        .await
        .unwrap();

    for _ in 0..100 {
        let (res, exp) = routing
            .execute_with_routing_and_fallback(
                "task-1",
                "prompt",
                m1.clone(),
                &available,
                &providers,
                &executor,
            )
            .await
            .unwrap();

        assert_eq!(res, baseline_res);
        assert_eq!(exp.primary_model_id, baseline_exp.primary_model_id);
        assert_eq!(exp.successful_model_id, baseline_exp.successful_model_id);
        assert_eq!(exp.attempts.len(), baseline_exp.attempts.len());

        for (i, attempt) in exp.attempts.iter().enumerate() {
            assert_eq!(attempt.model_id, baseline_exp.attempts[i].model_id);
            assert_eq!(attempt.error, baseline_exp.attempts[i].error);
        }
    }
}

#[test]
fn test_determinism_collaboration_strategy() {
    let collab = CollaborationService;

    let m1 = create_model(Uuid::now_v7(), 0.1);
    let m2 = create_model(Uuid::now_v7(), 0.1);
    let m3 = create_model(Uuid::now_v7(), 0.1);

    let available = vec![m1.clone(), m2.clone(), m3.clone()];

    let baseline_config = collab.determine_strategy(TaskType::Reasoning, m1.clone(), &available);

    for _ in 0..100 {
        let config = collab.determine_strategy(TaskType::Reasoning, m1.clone(), &available);

        assert_eq!(
            format!("{:?}", config.strategy),
            format!("{:?}", baseline_config.strategy)
        );
        assert_eq!(config.primary_model.id, baseline_config.primary_model.id);
        assert_eq!(
            config.secondary_models.len(),
            baseline_config.secondary_models.len()
        );

        for (i, secondary) in config.secondary_models.iter().enumerate() {
            assert_eq!(secondary.id, baseline_config.secondary_models[i].id);
        }
    }
}
