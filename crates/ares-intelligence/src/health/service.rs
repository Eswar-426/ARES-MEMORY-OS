use super::model_health::{ModelHealth, ModelStatus};
use super::provider_health::{ProviderHealth, ProviderStatus};
use crate::repository::health::HealthRepository;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

#[async_trait]
pub trait HealthManager: Send + Sync {
    async fn check_provider_health(&self, provider_id: &str) -> anyhow::Result<ProviderHealth>;
    async fn check_model_health(&self, model_id: Uuid) -> anyhow::Result<ModelHealth>;
    async fn report_provider_failure(&self, provider_id: &str) -> anyhow::Result<()>;
    async fn report_model_failure(&self, model_id: Uuid) -> anyhow::Result<()>;
}

#[allow(dead_code)]
pub struct HealthService {
    repo: Arc<dyn HealthRepository>,
}

impl HealthService {
    #[allow(dead_code)]
    pub fn new(repo: Arc<dyn HealthRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl HealthManager for HealthService {
    async fn check_provider_health(&self, provider_id: &str) -> anyhow::Result<ProviderHealth> {
        Ok(ProviderHealth {
            provider_id: provider_id.to_string(),
            status: ProviderStatus::Healthy,
            last_checked_at: chrono::Utc::now(),
            consecutive_failures: 0,
        })
    }

    async fn check_model_health(&self, model_id: Uuid) -> anyhow::Result<ModelHealth> {
        Ok(ModelHealth {
            model_id,
            status: ModelStatus::Healthy,
            last_checked_at: chrono::Utc::now(),
            consecutive_failures: 0,
        })
    }

    async fn report_provider_failure(&self, provider_id: &str) -> anyhow::Result<()> {
        self.repo.record_health_event(provider_id, "Failure").await
    }

    async fn report_model_failure(&self, _model_id: Uuid) -> anyhow::Result<()> {
        Ok(())
    }
}
