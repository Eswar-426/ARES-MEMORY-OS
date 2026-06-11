use crate::repository::intelligence_repository::{IntelligenceRepository, ProviderHealthEvent};
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;

pub struct HealthManager {
    repo: Arc<dyn IntelligenceRepository>,
}

impl HealthManager {
    pub fn new(repo: Arc<dyn IntelligenceRepository>) -> Self {
        Self { repo }
    }

    pub async fn update_health(
        &self,
        provider_id: &str,
        is_success: bool,
        latency_ms: i64,
    ) -> Result<()> {
        let _current_health = self.repo.get_provider_health(provider_id).await?;

        let mut status = "healthy".to_string();

        if !is_success {
            status = "down".to_string();
        } else if latency_ms > 5000 {
            // arbitrary degraded threshold
            status = "degraded".to_string();
        }

        self.repo
            .save_provider_health(ProviderHealthEvent {
                provider_id: provider_id.to_string(),
                status,
                last_checked_at: Utc::now(),
            })
            .await
    }

    pub async fn calculate_health_score(&self, provider_id: &str) -> Result<u32> {
        let health = self.repo.get_provider_health(provider_id).await?;
        if let Some(h) = health {
            match h.status.as_str() {
                "healthy" => Ok(100),
                "degraded" => Ok(50),
                "down" => Ok(0),
                _ => Ok(50),
            }
        } else {
            // Default to 100 if unknown
            Ok(100)
        }
    }
}
