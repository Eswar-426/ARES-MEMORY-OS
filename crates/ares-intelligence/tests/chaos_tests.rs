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
use std::sync::atomic::{AtomicUsize, Ordering};
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

// A chaos provider that alters its behavior based on how many times it's been called
pub struct ChaosProvider {
    pub name: String,
    pub call_count: AtomicUsize,
}

#[async_trait::async_trait]
impl ModelProvider for ChaosProvider {
    fn provider_id(&self) -> &str {
        &self.name
    }

    async fn health_check(&self) -> anyhow::Result<bool> {
        Ok(true)
    }

    async fn generate(&self, _prompt: &str) -> anyhow::Result<String> {
        let count = self.call_count.fetch_add(1, Ordering::SeqCst);

        match count {
            0 => Ok("Initial Success".to_string()),
            1 => anyhow::bail!("Suddenly offline!"), // Dies halfway
            2 => Ok("!@#$%^&*() Garbage Data".to_string()), // Returns garbage
            3 => {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                Ok("Late response".to_string()) // Becomes extremely slow
            }
            _ => anyhow::bail!("Permanent failure"),
        }
    }
}

#[tokio::test]
async fn test_chaos_provider_dies_halfway() {
    let fallback_manager = FallbackManager::new();
    let routing = RoutingService::new(fallback_manager);
    let executor = SandboxExecutor::new(Limits::default(), Quotas::default());

    let m1 = create_model(Uuid::now_v7(), "chaos");
    let m2 = create_model(Uuid::now_v7(), "stable");

    let mut providers: HashMap<String, Arc<dyn ModelProvider>> = HashMap::new();
    providers.insert(
        "chaos".to_string(),
        Arc::new(ChaosProvider {
            name: "chaos".to_string(),
            call_count: AtomicUsize::new(0),
        }),
    );
    providers.insert(
        "stable".to_string(),
        Arc::new(MockProvider::new(
            "stable",
            MockProviderBehavior::Success("Stable fallback".to_string()),
        )),
    );

    // Call 0: Success
    let (res, _) = routing
        .execute_with_routing_and_fallback(
            "t1",
            "prompt",
            m1.clone(),
            &[m1.clone(), m2.clone()],
            &providers,
            &executor,
        )
        .await
        .unwrap();
    assert_eq!(res, "Initial Success");

    // Call 1: Dies halfway. Should trigger fallback to m2
    let (res, exp) = routing
        .execute_with_routing_and_fallback(
            "t2",
            "prompt",
            m1.clone(),
            &[m1.clone(), m2.clone()],
            &providers,
            &executor,
        )
        .await
        .unwrap();
    assert_eq!(res, "Stable fallback");
    assert_eq!(exp.successful_model_id, m2.id.to_string());

    // Call 2: Returns garbage
    let (res, _) = routing
        .execute_with_routing_and_fallback(
            "t3",
            "prompt",
            m1.clone(),
            &[m1.clone(), m2.clone()],
            &providers,
            &executor,
        )
        .await
        .unwrap();
    assert_eq!(res, "!@#$%^&*() Garbage Data");
}
