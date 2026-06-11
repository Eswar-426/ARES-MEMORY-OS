use ares_intelligence::collaboration::service::CollaborationService;
use ares_intelligence::collaboration::strategy::CollaborationStrategy;
use ares_intelligence::ensemble::service::EnsembleService;
use ares_intelligence::models::capability::{ModelCapability, TaskType};
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

fn setup() -> (
    CollaborationService,
    RoutingService,
    EnsembleService,
    SandboxExecutor,
) {
    let collab = CollaborationService::new();
    let routing = RoutingService::new(FallbackManager::new());
    let ensemble = EnsembleService::default();
    let executor = SandboxExecutor::new(Limits::default(), Quotas::default());
    (collab, routing, ensemble, executor)
}

#[test]
fn test_determine_strategy_coding_single_model() {
    let (collab, _, _, _) = setup();
    let m1 = create_model(Uuid::now_v7(), "p1");
    // Only 1 model available -> SingleModel fallback
    let config = collab.determine_strategy(TaskType::Coding, m1.clone(), std::slice::from_ref(&m1));
    assert!(matches!(
        config.strategy,
        CollaborationStrategy::SingleModel
    ));
}

#[test]
fn test_determine_strategy_coding_reason_verify() {
    let (collab, _, _, _) = setup();
    let m1 = create_model(Uuid::now_v7(), "p1");
    let m2 = create_model(Uuid::now_v7(), "p2");

    // Multiple models -> ReasonAndVerify
    let config = collab.determine_strategy(TaskType::Coding, m1.clone(), &[m1.clone(), m2.clone()]);
    assert!(matches!(
        config.strategy,
        CollaborationStrategy::ReasonAndVerify
    ));
    assert_eq!(config.secondary_models.len(), 1);
}

#[test]
fn test_determine_strategy_research_debate() {
    let (collab, _, _, _) = setup();
    let m1 = create_model(Uuid::now_v7(), "p1");
    let m2 = create_model(Uuid::now_v7(), "p2");
    let m3 = create_model(Uuid::now_v7(), "p3");

    // Multiple models -> Debate
    let config = collab.determine_strategy(
        TaskType::Research,
        m1.clone(),
        &[m1.clone(), m2.clone(), m3.clone()],
    );
    assert!(matches!(config.strategy, CollaborationStrategy::Debate));
    assert_eq!(config.secondary_models.len(), 2);
}

#[tokio::test]
async fn test_execution_reason_and_verify_success() {
    let (collab, routing, ensemble, executor) = setup();
    let m1 = create_model(Uuid::now_v7(), "p1");
    let m2 = create_model(Uuid::now_v7(), "p2");

    let mut providers: HashMap<String, Arc<dyn ModelProvider>> = HashMap::new();
    providers.insert(
        "p1".to_string(),
        Arc::new(MockProvider::new(
            "p1",
            MockProviderBehavior::Success("Original Answer".to_string()),
        )),
    );
    providers.insert(
        "p2".to_string(),
        Arc::new(MockProvider::new(
            "p2",
            MockProviderBehavior::Success("Looks good to me".to_string()),
        )),
    ); // Verifier approves

    let res = collab
        .execute_collaboration(
            "task-1",
            TaskType::Coding,
            "prompt",
            m1.clone(),
            &[m1.clone(), m2.clone()],
            &routing,
            &ensemble,
            &providers,
            &executor,
        )
        .await
        .unwrap();

    assert_eq!(res, "Original Answer");
}

#[tokio::test]
async fn test_execution_reason_and_verify_rejection() {
    let (collab, routing, ensemble, executor) = setup();
    let m1 = create_model(Uuid::now_v7(), "p1");
    let m2 = create_model(Uuid::now_v7(), "p2");

    let mut providers: HashMap<String, Arc<dyn ModelProvider>> = HashMap::new();
    providers.insert(
        "p1".to_string(),
        Arc::new(MockProvider::new(
            "p1",
            MockProviderBehavior::Success("Bad code".to_string()),
        )),
    );
    providers.insert(
        "p2".to_string(),
        Arc::new(MockProvider::new(
            "p2",
            MockProviderBehavior::Success("I REJECT this answer".to_string()),
        )),
    ); // Verifier REJECTS

    let res = collab
        .execute_collaboration(
            "task-1",
            TaskType::Coding,
            "prompt",
            m1.clone(),
            &[m1.clone(), m2.clone()],
            &routing,
            &ensemble,
            &providers,
            &executor,
        )
        .await;

    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("Verification rejected"));
}

#[tokio::test]
async fn test_execution_debate_consensus() {
    let (collab, routing, ensemble, executor) = setup();
    let m1 = create_model(Uuid::now_v7(), "p1");
    let m2 = create_model(Uuid::now_v7(), "p2");
    let m3 = create_model(Uuid::now_v7(), "p3");

    let mut providers: HashMap<String, Arc<dyn ModelProvider>> = HashMap::new();
    providers.insert(
        "p1".to_string(),
        Arc::new(MockProvider::new(
            "p1",
            MockProviderBehavior::Success("Answer A".to_string()),
        )),
    );
    providers.insert(
        "p2".to_string(),
        Arc::new(MockProvider::new(
            "p2",
            MockProviderBehavior::Success("Answer A".to_string()),
        )),
    ); // M2 agrees with M1
    providers.insert(
        "p3".to_string(),
        Arc::new(MockProvider::new(
            "p3",
            MockProviderBehavior::Success("Answer B".to_string()),
        )),
    ); // M3 disagrees

    let res = collab
        .execute_collaboration(
            "task-1",
            TaskType::Research,
            "prompt",
            m1.clone(),
            &[m1.clone(), m2.clone(), m3.clone()],
            &routing,
            &ensemble,
            &providers,
            &executor,
        )
        .await
        .unwrap();

    // Aggregator should pick Answer A because 2 out of 3 said "Answer A"
    assert_eq!(res, "Answer A");
}

#[tokio::test]
async fn test_execution_debate_partial_group_failure() {
    let (collab, routing, ensemble, executor) = setup();
    let m1 = create_model(Uuid::now_v7(), "p1");
    let m2 = create_model(Uuid::now_v7(), "p2");
    let m3 = create_model(Uuid::now_v7(), "p3");

    let mut providers: HashMap<String, Arc<dyn ModelProvider>> = HashMap::new();
    providers.insert(
        "p1".to_string(),
        Arc::new(MockProvider::new(
            "p1",
            MockProviderBehavior::Success("Final Answer".to_string()),
        )),
    );
    providers.insert(
        "p2".to_string(),
        Arc::new(MockProvider::new("p2", MockProviderBehavior::Offline)),
    ); // M2 fails
    providers.insert(
        "p3".to_string(),
        Arc::new(MockProvider::new(
            "p3",
            MockProviderBehavior::Success("Final Answer".to_string()),
        )),
    ); // M3 succeeds

    let res = collab
        .execute_collaboration(
            "task-1",
            TaskType::Research,
            "prompt",
            m1.clone(),
            &[m1.clone(), m2.clone(), m3.clone()],
            &routing,
            &ensemble,
            &providers,
            &executor,
        )
        .await
        .unwrap();

    // Even though M2 failed, M1 and M3 succeeded, so the ensemble should gracefully succeed.
    assert_eq!(res, "Final Answer");
}

#[tokio::test]
async fn test_execution_debate_total_failure() {
    let (collab, routing, ensemble, executor) = setup();
    let m1 = create_model(Uuid::now_v7(), "p1");
    let m2 = create_model(Uuid::now_v7(), "p2");

    let mut providers: HashMap<String, Arc<dyn ModelProvider>> = HashMap::new();
    providers.insert(
        "p1".to_string(),
        Arc::new(MockProvider::new("p1", MockProviderBehavior::Offline)),
    );
    providers.insert(
        "p2".to_string(),
        Arc::new(MockProvider::new("p2", MockProviderBehavior::RateLimit)),
    );

    let res = collab
        .execute_collaboration(
            "task-1",
            TaskType::Research,
            "prompt",
            m1.clone(),
            &[m1.clone(), m2.clone()],
            &routing,
            &ensemble,
            &providers,
            &executor,
        )
        .await;

    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("Total ensemble failure"));
}
