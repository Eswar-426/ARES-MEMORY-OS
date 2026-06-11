use super::engine::PolicyEngine;
use super::rules::PolicySet;
use crate::models::model::Model;
use crate::models::profile::ModelProfile;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait PolicyManager: Send + Sync {
    async fn is_model_allowed(
        &self,
        model: &Model,
        profile: Option<&ModelProfile>,
        policy_set: &PolicySet,
    ) -> anyhow::Result<bool>;
}

#[allow(dead_code)]
pub struct PolicyService {
    engine: Arc<PolicyEngine>,
}

impl PolicyService {
    #[allow(dead_code)]
    pub fn new(engine: Arc<PolicyEngine>) -> Self {
        Self { engine }
    }
}

#[async_trait]
impl PolicyManager for PolicyService {
    async fn is_model_allowed(
        &self,
        model: &Model,
        profile: Option<&ModelProfile>,
        policy_set: &PolicySet,
    ) -> anyhow::Result<bool> {
        self.engine.evaluate_model(model, profile, policy_set)
    }
}
