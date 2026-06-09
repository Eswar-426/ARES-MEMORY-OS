use crate::capability_cache::models::{CachedCapability, CachedModel};
use ares_core::AresError;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

const CACHE_TTL: Duration = Duration::from_secs(60);

pub struct CapabilityCacheService {
    capabilities: Arc<RwLock<HashMap<String, CachedCapability>>>,
    models: Arc<RwLock<HashMap<String, CachedModel>>>,
}

impl CapabilityCacheService {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            capabilities: Arc::new(RwLock::new(HashMap::new())),
            models: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_capability(&self, name: &str) -> Result<Option<CachedCapability>, AresError> {
        let cache = self.capabilities.read().unwrap();
        if let Some(cap) = cache.get(name) {
            let age = Utc::now()
                .signed_duration_since(cap.cached_at)
                .to_std()
                .unwrap_or(Duration::from_secs(0));
            if age <= CACHE_TTL {
                return Ok(Some(cap.clone()));
            }
        }
        Ok(None)
    }

    pub fn set_capability(&self, cap: CachedCapability) -> Result<(), AresError> {
        let mut cache = self.capabilities.write().unwrap();
        cache.insert(cap.name.clone(), cap);
        Ok(())
    }

    pub fn invalidate_all(&self) {
        self.capabilities.write().unwrap().clear();
        self.models.write().unwrap().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_cache_miss() {
        let service = CapabilityCacheService::new();
        let cap = service.get_capability("non_existent").unwrap();
        assert!(cap.is_none());
    }

    #[test]
    fn test_cache_hit_and_invalidate() {
        let service = CapabilityCacheService::new();

        let cap = CachedCapability {
            name: "test_cap".to_string(),
            description: "Test".to_string(),
            required_inputs: vec![],
            outputs: vec![],
            provider_agent_id: ares_core::id::AgentId::new(),
            cached_at: Utc::now(),
        };

        service.set_capability(cap.clone()).unwrap();

        let hit = service.get_capability("test_cap").unwrap();
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().name, "test_cap");

        service.invalidate_all();
        let miss = service.get_capability("test_cap").unwrap();
        assert!(miss.is_none());
    }

    #[test]
    fn test_cache_ttl_expiration() {
        let service = CapabilityCacheService::new();

        // Simulating a deeply expired timestamp
        let expired_time = Utc::now() - chrono::Duration::seconds(100);
        let cap = CachedCapability {
            name: "expired_cap".to_string(),
            description: "Test".to_string(),
            required_inputs: vec![],
            outputs: vec![],
            provider_agent_id: ares_core::id::AgentId::new(),
            cached_at: expired_time,
        };

        service.set_capability(cap).unwrap();

        let hit = service.get_capability("expired_cap").unwrap();
        // Since TTL is 60s, a 100s old cache entry should return None
        assert!(hit.is_none());
    }
}
