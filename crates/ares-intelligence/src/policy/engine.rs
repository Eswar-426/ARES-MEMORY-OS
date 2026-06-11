use super::rules::{PolicyRule, PolicySet};
use crate::models::model::Model;
use crate::models::profile::ModelProfile;

pub struct PolicyEngine;

impl PolicyEngine {
    pub fn evaluate_model(
        &self,
        model: &Model,
        profile: Option<&ModelProfile>,
        policy_set: &PolicySet,
    ) -> anyhow::Result<bool> {
        for rule in &policy_set.rules {
            match rule {
                PolicyRule::AllowedProviders(providers) => {
                    if !providers.contains(&model.provider_id) {
                        return Ok(false);
                    }
                }
                PolicyRule::ForbiddenModels(models) => {
                    if models.contains(&model.id.to_string()) {
                        return Ok(false);
                    }
                }
                PolicyRule::MaxLatencyMs(max_latency) => {
                    if let Some(p) = profile {
                        if p.average_latency_ms > *max_latency {
                            return Ok(false);
                        }
                    }
                }
                PolicyRule::RequiredConfidence(req_conf) => {
                    if let Some(p) = profile {
                        if p.success_rate < *req_conf {
                            return Ok(false);
                        }
                    }
                }
                PolicyRule::MaxCostPerRequest(_cost) => {
                    // Requires cost estimation logic which isn't available yet
                }
            }
        }
        Ok(true)
    }
}
