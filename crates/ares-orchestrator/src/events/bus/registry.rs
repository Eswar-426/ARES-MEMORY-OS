use super::r#trait::EventBus;
use std::sync::Arc;

pub struct EventBusRegistry {
    active_bus: Arc<dyn EventBus>,
}

impl EventBusRegistry {
    pub fn new(active_bus: Arc<dyn EventBus>) -> Self {
        Self { active_bus }
    }

    pub fn get(&self) -> Arc<dyn EventBus> {
        self.active_bus.clone()
    }
}
