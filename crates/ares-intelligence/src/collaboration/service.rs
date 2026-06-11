use super::strategy::{CollaborationConfig, CollaborationStrategy};
use crate::ensemble::service::EnsembleService;
use crate::models::capability::TaskType;
use crate::models::model::Model;
use crate::providers::ModelProvider;
use crate::routing::service::RoutingService;
use crate::sandbox::executor::SandboxExecutor;
use std::collections::HashMap;
use std::sync::Arc;

pub struct CollaborationService;

impl Default for CollaborationService {
    fn default() -> Self {
        Self::new()
    }
}

impl CollaborationService {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    pub fn determine_strategy(
        &self,
        task_type: TaskType,
        selected_model: Model,
        available_models: &[Model],
    ) -> CollaborationConfig {
        match task_type {
            TaskType::Coding | TaskType::Reasoning => {
                let secondary: Vec<Model> = available_models
                    .iter()
                    .filter(|m| m.id != selected_model.id)
                    .take(1)
                    .cloned()
                    .collect();

                CollaborationConfig {
                    strategy: if secondary.is_empty() {
                        CollaborationStrategy::SingleModel
                    } else {
                        CollaborationStrategy::ReasonAndVerify
                    },
                    primary_model: selected_model,
                    secondary_models: secondary,
                    max_rounds: 2,
                }
            }
            TaskType::Research => {
                let secondary: Vec<Model> = available_models
                    .iter()
                    .filter(|m| m.id != selected_model.id)
                    .take(2)
                    .cloned()
                    .collect();

                CollaborationConfig {
                    strategy: if secondary.is_empty() {
                        CollaborationStrategy::SingleModel
                    } else {
                        CollaborationStrategy::Debate
                    },
                    primary_model: selected_model,
                    secondary_models: secondary,
                    max_rounds: 3,
                }
            }
            _ => CollaborationConfig {
                strategy: CollaborationStrategy::SingleModel,
                primary_model: selected_model,
                secondary_models: Vec::new(),
                max_rounds: 1,
            },
        }
    }

    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub async fn execute_collaboration(
        &self,
        _task_id: &str,
        task_type: TaskType,
        prompt: &str,
        selected_model: Model,
        available_models: &[Model],
        routing_service: &RoutingService,
        ensemble_service: &EnsembleService,
        _providers: &HashMap<String, Arc<dyn ModelProvider>>,
        _executor: &SandboxExecutor,
    ) -> anyhow::Result<String> {
        let config = self.determine_strategy(task_type, selected_model.clone(), available_models);

        match config.strategy {
            CollaborationStrategy::SingleModel => {
                let res = routing_service
                    .execute(&selected_model.id.to_string(), prompt)
                    .await
                    .map_err(|e| anyhow::anyhow!("Routing error: {:?}", e))?;
                Ok(res.content)
            }
            CollaborationStrategy::ReasonAndVerify => {
                let primary_res = routing_service
                    .execute(&config.primary_model.id.to_string(), prompt)
                    .await
                    .map_err(|e| anyhow::anyhow!("Routing error: {:?}", e))?
                    .content;

                if let Some(verifier) = config.secondary_models.first() {
                    let verify_prompt = format!(
                        "Verify this output:\n{}\n\nOriginal Prompt:\n{}",
                        primary_res, prompt
                    );
                    let verify_res = routing_service
                        .execute(&verifier.id.to_string(), &verify_prompt)
                        .await
                        .map_err(|e| anyhow::anyhow!("Routing error: {:?}", e))?
                        .content;

                    if verify_res.contains("REJECT") {
                        anyhow::bail!("Verification rejected the response");
                    }
                }

                Ok(primary_res)
            }
            CollaborationStrategy::Debate => {
                let mut tasks = Vec::new();
                let mut all_models = config.secondary_models.clone();
                all_models.push(config.primary_model);

                for model in all_models {
                    let routing = routing_service;
                    let p = prompt.to_string();
                    // We must not pass Arc<RoutingService>, instead we'll just execute sequentially for simplicity since it's a test architecture
                    // Actually, let's just await sequentially instead of tokio::spawn to avoid lifetime issues with references!

                    let res = routing
                        .execute(&model.id.to_string(), &p)
                        .await
                        .map(|resp| resp.content)
                        .map_err(|e| anyhow::anyhow!("Routing error: {:?}", e));

                    tasks.push(res);
                }

                let mut successful_responses = Vec::new();
                for answer in tasks.into_iter().flatten() {
                    successful_responses.push(answer);
                }

                if successful_responses.is_empty() {
                    anyhow::bail!("Total ensemble failure: all models failed in debate");
                }

                let (resolved, _) = ensemble_service.resolve_conflict(&successful_responses)?;
                Ok(resolved)
            }
            _ => {
                anyhow::bail!("Strategy not implemented for execution");
            }
        }
    }
}
