use anyhow::{anyhow, Result};
use ares_intelligence::cost::anomaly::CostAnomalyDetector;
use ares_intelligence::cost::budget_manager::BudgetManager;
use ares_intelligence::cost::kill_switch::KillSwitch;
use ares_intelligence::cost::provider_quota::{ProviderQuotaConfig, ProviderQuotaManager};
use ares_intelligence::reliability::circuit_breaker::{BreakerState, CircuitBreaker};
use ares_intelligence::reliability::retry::RetryEngine;
use ares_intelligence::repository::intelligence_repository::{
    CircuitBreakerState, CostEvent, ExecutionTrace, IntelligenceRepository, LearningEvent,
    ProviderHealthEvent, RoutingDecision, SelectionExplanation,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Mock Repo for tests
struct MockIntelligenceRepo;

#[async_trait::async_trait]
impl IntelligenceRepository for MockIntelligenceRepo {
    async fn save_selection_explanation(&self, _exp: SelectionExplanation) -> Result<()> {
        Ok(())
    }
    async fn save_routing_decision(&self, _dec: RoutingDecision) -> Result<()> {
        Ok(())
    }
    async fn save_execution_trace(&self, _trace: ExecutionTrace) -> Result<()> {
        Ok(())
    }
    async fn save_cost_event(&self, _event: CostEvent) -> Result<()> {
        Ok(())
    }
    async fn save_learning_event(&self, _event: LearningEvent) -> Result<()> {
        Ok(())
    }
    async fn save_provider_health(&self, _health: ProviderHealthEvent) -> Result<()> {
        Ok(())
    }

    async fn get_provider_health(&self, _id: &str) -> Result<Option<ProviderHealthEvent>> {
        Ok(None)
    }

    async fn save_circuit_breaker_state(&self, _state: CircuitBreakerState) -> Result<()> {
        Ok(())
    }

    async fn get_circuit_breaker_state(
        &self,
        provider_id: &str,
    ) -> Result<Option<CircuitBreakerState>> {
        // Return dummy open state for test if provider_id == "broken"
        if provider_id == "broken" {
            return Ok(Some(CircuitBreakerState {
                provider_id: provider_id.to_string(),
                state: "open".to_string(),
                failure_count: 5,
                opened_at: Some(chrono::Utc::now()),
            }));
        }
        Ok(None)
    }
}

#[tokio::test]
async fn test_circuit_breaker_open_state() {
    let repo = Arc::new(MockIntelligenceRepo);
    let breaker = CircuitBreaker::new(repo, 5, 10000);

    let state = breaker.check("broken").await.unwrap();
    assert!(matches!(state, BreakerState::Open));
}

#[tokio::test]
async fn test_budget_manager() {
    let manager = BudgetManager::new(100.0);
    manager.add_spend(50.0);
    assert!(!manager.is_budget_exceeded());
    manager.add_spend(50.1);
    assert!(manager.is_budget_exceeded());
}

#[tokio::test]
async fn test_cost_anomaly_detection() {
    let detector = CostAnomalyDetector::new(10.0, 5.0); // anything > 50 is anomaly
    detector.record_spend(20.0);
    assert!(!detector.detect_anomaly());
    detector.record_spend(40.0);
    assert!(detector.detect_anomaly());
}

#[tokio::test]
async fn test_retry_engine_exhaustion() {
    let engine = RetryEngine::new(2, 10);
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_clone = attempts.clone();

    let result = engine
        .execute_with_retry(|| {
            let attempts = attempts_clone.clone();
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(anyhow!("fail"))
            }
        })
        .await;

    assert!(result.is_err());
    assert_eq!(attempts.load(Ordering::SeqCst), 3); // initial + 2 retries
}

#[tokio::test]
async fn test_kill_switch() {
    let switch = KillSwitch::new();
    assert!(!switch.is_active());
    switch.activate();
    assert!(switch.is_active());
}

#[test]
fn test_provider_quota() {
    let configs = vec![ProviderQuotaConfig {
        provider_id: "openai".to_string(),
        max_requests_per_day: 5,
        max_tokens_per_day: 1000,
        max_spend_per_day: 1.0,
    }];

    let manager = ProviderQuotaManager::new(configs);

    manager.record_usage("openai", 500, 0.5);
    assert!(manager.check_quota("openai").is_ok());

    manager.record_usage("openai", 600, 0.6); // tokens > 1000, spend > 1.0

    let err = manager.check_quota("openai").unwrap_err();
    assert!(err.to_string().contains("exceeded"));
}
