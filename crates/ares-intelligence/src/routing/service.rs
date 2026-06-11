use super::fallback::FallbackManager;
use crate::explanations::models::{RoutingAttempt, RoutingExplanation};
use crate::models::model::Model;
use crate::providers::ModelProvider;
use crate::sandbox::executor::SandboxExecutor;
use std::collections::HashMap;
use std::sync::Arc;

pub struct RoutingService {
    #[allow(dead_code)]
    fallback_manager: FallbackManager,
}

impl Default for RoutingService {
    fn default() -> Self {
        Self::new(FallbackManager)
    }
}

impl RoutingService {
    pub fn new(fallback_manager: FallbackManager) -> Self {
        Self { fallback_manager }
    }

    pub async fn execute_with_routing_and_fallback(
        &self,
        task_id: &str,
        prompt: &str,
        primary_model: Model,
        available_models: &[Model],
        providers: &HashMap<String, Arc<dyn ModelProvider>>,
        executor: &SandboxExecutor,
    ) -> anyhow::Result<(String, RoutingExplanation)> {
        let mut attempts = Vec::new();
        let mut current_model = primary_model.clone();

        loop {
            let provider = providers.get(&current_model.provider_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "Provider not found for model: {}",
                    current_model.provider_id
                )
            })?;

            match executor.execute(provider.as_ref(), prompt).await {
                Ok(result) => {
                    attempts.push(RoutingAttempt {
                        model_id: current_model.id.to_string(),
                        error: None,
                    });
                    let explanation = RoutingExplanation {
                        task_id: task_id.to_string(),
                        primary_model_id: primary_model.id.to_string(),
                        successful_model_id: current_model.id.to_string(),
                        attempts,
                    };
                    return Ok((result, explanation));
                }
                Err(e) => {
                    attempts.push(RoutingAttempt {
                        model_id: current_model.id.to_string(),
                        error: Some(e.to_string()),
                    });

                    // Attempt fallback
                    let mut found_fallback = false;
                    for candidate in available_models {
                        // Skip models we've already attempted
                        if !attempts
                            .iter()
                            .any(|a| a.model_id == candidate.id.to_string())
                        {
                            current_model = candidate.clone();
                            found_fallback = true;
                            break;
                        }
                    }

                    if !found_fallback {
                        break; // Exhausted all fallbacks
                    }
                }
            }
        }

        let explanation = RoutingExplanation {
            task_id: task_id.to_string(),
            primary_model_id: primary_model.id.to_string(),
            successful_model_id: "".to_string(),
            attempts: attempts.clone(),
        };
        Err(anyhow::anyhow!(
            "Routing failed and all fallbacks exhausted. Explanation: {:?}",
            explanation
        ))
    }
}
