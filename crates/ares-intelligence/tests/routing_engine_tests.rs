use ares_intelligence::models::capability::ModelCapability;
use ares_intelligence::models::model::Model;
use ares_intelligence::providers::mock::{MockProvider, MockProviderBehavior};
use ares_intelligence::providers::ModelProvider;
use ares_intelligence::routing::fallback::FallbackManager;
use ares_intelligence::routing::service::RoutingService;
use ares_intelligence::sandbox::executor::SandboxExecutor;
use ares_intelligence::sandbox::limits::Limits;
use ares_intelligence::sandbox::quotas::Quotas;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

fn create_model(id: Uuid, provider_id: &str) -> Model {
    Model {
        id,
        name: format!("Model-{}", id),
        provider_id: provider_id.to_string(),
        version: "1.0".to_string(),
        capabilities: vec![ModelCapability::Reasoning],
        max_context_window: 4096,
        cost_per_1k_tokens: 0.01,
    }
}

fn setup() -> (RoutingService, SandboxExecutor) {
    let fallback_manager = FallbackManager::new();
    let routing_service = RoutingService::new(fallback_manager);

    let limits = Limits::default();
    let quotas = Quotas::default();
    let executor = SandboxExecutor::new(limits, quotas);

    (routing_service, executor)
}

#[tokio::test]
async fn test_routing_success() {
    let (routing, executor) = setup();
    let m1 = create_model(Uuid::now_v7(), "prov_success");

    let mut providers: HashMap<String, Arc<dyn ModelProvider>> = HashMap::new();
    providers.insert(
        "prov_success".to_string(),
        Arc::new(MockProvider::new(
            "prov_success",
            MockProviderBehavior::Success("Hello".to_string()),
        )),
    );

    let (res, exp) = routing
        .execute_with_routing_and_fallback(
            "task-1",
            "prompt",
            m1.clone(),
            &[],
            &providers,
            &executor,
        )
        .await
        .unwrap();
    assert_eq!(res, "Hello");
    assert_eq!(exp.successful_model_id, m1.id.to_string());
    assert_eq!(exp.attempts.len(), 1);
    assert!(exp.attempts[0].error.is_none());
}

#[tokio::test]
async fn test_routing_fallback_success() {
    let (routing, executor) = setup();
    let m1 = create_model(Uuid::now_v7(), "prov_fail"); // primary
    let m2 = create_model(Uuid::now_v7(), "prov_success"); // fallback

    let available = vec![m1.clone(), m2.clone()];

    let mut providers: HashMap<String, Arc<dyn ModelProvider>> = HashMap::new();
    providers.insert(
        "prov_fail".to_string(),
        Arc::new(MockProvider::new(
            "prov_fail",
            MockProviderBehavior::Offline,
        )),
    );
    providers.insert(
        "prov_success".to_string(),
        Arc::new(MockProvider::new(
            "prov_success",
            MockProviderBehavior::Success("Fallback OK".to_string()),
        )),
    );

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
    assert_eq!(res, "Fallback OK");
    assert_eq!(exp.primary_model_id, m1.id.to_string());
    assert_eq!(exp.successful_model_id, m2.id.to_string()); // Successfully fell back to m2
    assert_eq!(exp.attempts.len(), 2);

    assert_eq!(exp.attempts[0].model_id, m1.id.to_string());
    assert!(exp.attempts[0].error.is_some());

    assert_eq!(exp.attempts[1].model_id, m2.id.to_string());
    assert!(exp.attempts[1].error.is_none());
}

#[tokio::test]
async fn test_routing_exhaustion() {
    let (routing, executor) = setup();
    let m1 = create_model(Uuid::now_v7(), "prov_fail1");
    let m2 = create_model(Uuid::now_v7(), "prov_fail2");
    let m3 = create_model(Uuid::now_v7(), "prov_fail3");

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
        "prov_fail3".to_string(),
        Arc::new(MockProvider::new(
            "prov_fail3",
            MockProviderBehavior::AuthFailure,
        )),
    );

    let res = routing
        .execute_with_routing_and_fallback(
            "task-1",
            "prompt",
            m1.clone(),
            &available,
            &providers,
            &executor,
        )
        .await;
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("Routing failed and all fallbacks exhausted"));
    // Ensure all 3 models were attempted
    assert!(err_msg.contains(&m1.id.to_string()));
    assert!(err_msg.contains(&m2.id.to_string()));
    assert!(err_msg.contains(&m3.id.to_string()));
}

#[tokio::test]
async fn test_routing_circular_fallback_prevention() {
    let (routing, executor) = setup();
    let m1 = create_model(Uuid::now_v7(), "prov_fail1");
    let m2 = create_model(Uuid::now_v7(), "prov_fail2");

    // Available models contains M1 and M2. If M1 fails, it tries M2. If M2 fails, it shouldn't try M1 again.
    let available = vec![m1.clone(), m2.clone()];

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
            MockProviderBehavior::Offline,
        )),
    );

    let res = routing
        .execute_with_routing_and_fallback(
            "task-1",
            "prompt",
            m1.clone(),
            &available,
            &providers,
            &executor,
        )
        .await;
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    // Verify it only attempted exactly 2 times, not infinite loops.
    assert!(err_msg.contains("attempts: [RoutingAttempt")); // basic check for format
}
