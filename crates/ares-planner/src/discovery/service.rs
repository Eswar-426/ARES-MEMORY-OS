use crate::capability_cache::models::CachedCapability;
use crate::capability_cache::service::CapabilityCacheService;
use ares_core::AresError;
use std::sync::Arc;

pub struct DiscoveryService {
    cache: Arc<CapabilityCacheService>,
}

impl DiscoveryService {
    pub fn new(cache: Arc<CapabilityCacheService>) -> Self {
        Self { cache }
    }

    /// Discovers a capability. Uses the local cache first, then falls back to `ares-knowledge`.
    /// Enforces Rule 5: No direct agent queries.
    pub fn find_capability(
        &self,
        required_capability: &str,
    ) -> Result<CachedCapability, AresError> {
        // 1. Check Cache
        if let Some(cap) = self.cache.get_capability(required_capability)? {
            return Ok(cap);
        }

        // 2. Cache miss. Query Knowledge Graph
        // Placeholder for `ares-knowledge` Graph API call.
        // For now, we return a simulated response and cache it.
        let discovered = CachedCapability {
            name: required_capability.to_string(),
            description: "Discovered via KG".into(),
            required_inputs: vec![],
            outputs: vec![],
            provider_agent_id: ares_core::id::AgentId::new(),
            cached_at: chrono::Utc::now(),
        };

        self.cache.set_capability(discovered.clone())?;

        Ok(discovered)
    }
}
