use super::types::{ProviderHealthStatus, ProviderMetadata};
use super::ModelProvider;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ProviderRegistry {
    providers: RwLock<HashMap<String, Arc<dyn ModelProvider>>>,
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(&self, provider: Arc<dyn ModelProvider>) {
        let metadata = provider.metadata();
        let mut w = self.providers.write().await;
        w.insert(metadata.id.clone(), provider);
    }

    pub async fn remove(&self, id: &str) -> Option<Arc<dyn ModelProvider>> {
        let mut w = self.providers.write().await;
        w.remove(id)
    }

    pub async fn get(&self, id: &str) -> Option<Arc<dyn ModelProvider>> {
        let r = self.providers.read().await;
        r.get(id).cloned()
    }

    pub async fn get_health(&self, id: &str) -> Option<ProviderHealthStatus> {
        let provider = self.get(id).await?;
        Some(provider.health_check().await)
    }

    pub async fn discover_healthy(&self) -> Vec<Arc<dyn ModelProvider>> {
        let mut healthy_providers = Vec::new();
        let providers_copy: Vec<Arc<dyn ModelProvider>> = {
            let r = self.providers.read().await;
            r.values().cloned().collect()
        };

        for provider in providers_copy {
            if provider.health_check().await == ProviderHealthStatus::Healthy {
                healthy_providers.push(provider);
            }
        }
        healthy_providers
    }

    pub async fn get_metadata_all(&self) -> Vec<ProviderMetadata> {
        let r = self.providers.read().await;
        r.values().map(|p| p.metadata()).collect()
    }
}
