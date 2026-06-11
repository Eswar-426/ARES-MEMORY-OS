use crate::explanations::models::{RejectedModel, SelectionExplanation};
use crate::models::capability::{ModelCapability, TaskType};
use crate::models::model::Model;
use crate::models::profile::ModelProfile;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Objective {
    Fastest,
    Cheapest,
    HighestQuality,
    Balanced,
}

pub struct SelectionCriteria {
    pub task_id: String,
    pub task_type: TaskType,
    pub required_caps: Vec<ModelCapability>,
    pub objective: Objective,
    pub max_latency_ms: Option<u64>,
    pub max_cost: Option<f64>,
}

pub struct ModelSelector;

impl Default for ModelSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelSelector {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    pub fn select_best_model(
        &self,
        criteria: &SelectionCriteria,
        available_models: &[Model],
        profiles: &HashMap<uuid::Uuid, ModelProfile>,
    ) -> anyhow::Result<(Model, SelectionExplanation)> {
        let mut rejected = Vec::new();
        let mut candidates = Vec::new();

        for model in available_models {
            let has_all = criteria
                .required_caps
                .iter()
                .all(|cap| model.capabilities.contains(cap));
            if !has_all {
                rejected.push(RejectedModel {
                    model_id: model.id.to_string(),
                    reason: "Missing required capabilities".to_string(),
                });
                continue;
            }

            if let Some(profile) = profiles.get(&model.id) {
                if let Some(max_latency) = criteria.max_latency_ms {
                    if profile.average_latency_ms > max_latency {
                        rejected.push(RejectedModel {
                            model_id: model.id.to_string(),
                            reason: format!(
                                "Average latency {} exceeds limit {}",
                                profile.average_latency_ms, max_latency
                            ),
                        });
                        continue;
                    }
                }
            }

            if let Some(max_cost) = criteria.max_cost {
                if model.cost_per_1k_tokens > max_cost {
                    rejected.push(RejectedModel {
                        model_id: model.id.to_string(),
                        reason: format!(
                            "Cost {} exceeds limit {}",
                            model.cost_per_1k_tokens, max_cost
                        ),
                    });
                    continue;
                }
            }

            candidates.push(model.clone());
        }

        if candidates.is_empty() {
            anyhow::bail!("No suitable model found for task");
        }

        candidates.sort_by(|a, b| {
            let p_a = profiles.get(&a.id);
            let p_b = profiles.get(&b.id);

            let cmp = match criteria.objective {
                Objective::Cheapest => a
                    .cost_per_1k_tokens
                    .partial_cmp(&b.cost_per_1k_tokens)
                    .unwrap(),
                Objective::Fastest => {
                    let lat_a = p_a.map(|p| p.average_latency_ms).unwrap_or(u64::MAX);
                    let lat_b = p_b.map(|p| p.average_latency_ms).unwrap_or(u64::MAX);
                    lat_a.cmp(&lat_b)
                }
                Objective::HighestQuality => {
                    let q_a = p_a.map(|p| p.success_rate).unwrap_or(0.0);
                    let q_b = p_b.map(|p| p.success_rate).unwrap_or(0.0);
                    q_b.partial_cmp(&q_a).unwrap()
                }
                Objective::Balanced => {
                    let score_a = p_a.map(|p| p.success_rate).unwrap_or(0.5)
                        / (a.cost_per_1k_tokens.max(0.001));
                    let score_b = p_b.map(|p| p.success_rate).unwrap_or(0.5)
                        / (b.cost_per_1k_tokens.max(0.001));
                    score_b.partial_cmp(&score_a).unwrap()
                }
            };
            if cmp == std::cmp::Ordering::Equal {
                a.id.cmp(&b.id)
            } else {
                cmp
            }
        });

        let best = candidates[0].clone();

        let explanation = SelectionExplanation {
            task_id: criteria.task_id.clone(),
            selected_model_id: best.id.to_string(),
            required_capabilities: criteria.required_caps.clone(),
            reasoning: format!("Selected using objective {:?}", criteria.objective),
            rejected_models: rejected,
        };

        Ok((best, explanation))
    }
}
