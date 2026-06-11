use crate::catalog::ModelCatalog;
use crate::cost::budget_manager::BudgetManager;
use crate::providers::registry::ProviderRegistry;
use crate::providers::{
    types::{ModelRequest, ModelResponse},
    ProviderError,
};
use crate::reliability::circuit_breaker::{BreakerState, CircuitBreaker};
use crate::reliability::quota_manager::QuotaManager;
use crate::reliability::retry::RetryEngine;
use crate::tokens::TokenEstimator;
use std::sync::Arc;

pub struct RoutingService {
    registry: Arc<ProviderRegistry>,
    circuit_breaker: Arc<CircuitBreaker>,
    retry_engine: Arc<RetryEngine>,
    quota_manager: Arc<QuotaManager>,
    budget_manager: Arc<BudgetManager>,
    catalog: Arc<ModelCatalog>,
}

impl RoutingService {
    pub fn new(
        registry: Arc<ProviderRegistry>,
        circuit_breaker: Arc<CircuitBreaker>,
        retry_engine: Arc<RetryEngine>,
        quota_manager: Arc<QuotaManager>,
        budget_manager: Arc<BudgetManager>,
        catalog: Arc<ModelCatalog>,
    ) -> Self {
        Self {
            registry,
            circuit_breaker,
            retry_engine,
            quota_manager,
            budget_manager,
            catalog,
        }
    }

    pub async fn execute(
        &self,
        model_id: &str,
        prompt: &str,
    ) -> Result<ModelResponse, ProviderError> {
        let model_entry = self
            .catalog
            .get(model_id)
            .ok_or_else(|| ProviderError::Unknown("Model not found in catalog".to_string()))?;

        let provider = self
            .registry
            .get(&model_entry.provider_id)
            .await
            .ok_or(ProviderError::ProviderUnavailable)?;

        // Pre-flight Checks
        // 1. Quota
        if !self
            .quota_manager
            .check_and_consume(&model_entry.provider_id)
            .await
        {
            return Err(ProviderError::RateLimited);
        }

        // 2. Budget
        let est_cost =
            TokenEstimator::estimate_prompt_cost(prompt, model_entry.cost_per_input_token);
        if !self.budget_manager.check_request_budget(est_cost) {
            return Err(ProviderError::BudgetExceeded);
        }

        // 3. Circuit Breaker
        match self.circuit_breaker.check(&model_entry.provider_id).await {
            Ok(BreakerState::Open) => return Err(ProviderError::CircuitOpen),
            Err(_) => return Err(ProviderError::Unknown("Circuit breaker failed".to_string())),
            _ => {}
        }

        let request = ModelRequest {
            prompt: prompt.to_string(),
            max_tokens: None,
            temperature: None,
            stream: false,
        };

        // 4. Execution with Retry
        let result = self
            .retry_engine
            .execute_with_retry(|| async { provider.generate(request.clone()).await })
            .await;

        match result {
            Ok(resp) => {
                let _ = self
                    .circuit_breaker
                    .record_success(&model_entry.provider_id)
                    .await;
                // Add actual spend based on resp tokens
                let total_cost = (resp.prompt_tokens as f64 * model_entry.cost_per_input_token)
                    + (resp.completion_tokens as f64 * model_entry.cost_per_output_token);
                self.budget_manager.add_spend(total_cost);
                Ok(resp)
            }
            Err(e) => {
                match e {
                    ProviderError::Timeout
                    | ProviderError::ConnectionFailed
                    | ProviderError::ProviderUnavailable => {
                        let _ = self
                            .circuit_breaker
                            .record_failure(&model_entry.provider_id)
                            .await;
                    }
                    _ => {}
                }
                Err(e)
            }
        }
    }
}
