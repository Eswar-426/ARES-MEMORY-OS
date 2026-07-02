use crate::core::capabilities::Capability;
use crate::core::engine::{EngineId, RepositoryEngine};
use std::collections::HashMap;

pub struct EngineRegistry {
    engines: HashMap<EngineId, Box<dyn RepositoryEngine>>,
    capability_map: HashMap<Capability, Vec<EngineId>>,
}

impl Default for EngineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineRegistry {
    pub fn new() -> Self {
        Self {
            engines: HashMap::new(),
            capability_map: HashMap::new(),
        }
    }

    pub fn resolve_capabilities(&self, capability: &Capability) -> Vec<EngineId> {
        self.capability_map
            .get(capability)
            .cloned()
            .unwrap_or_default()
    }

    pub fn register(
        &mut self,
        id: EngineId,
        capabilities: Vec<Capability>,
        engine: Box<dyn RepositoryEngine>,
    ) {
        for cap in capabilities {
            self.capability_map.entry(cap).or_default().push(id.clone());
        }
        self.engines.insert(id, engine);
    }

    pub fn get_engine(&self, id: &EngineId) -> Option<&dyn RepositoryEngine> {
        self.engines.get(id).map(|e| e.as_ref())
    }
}
