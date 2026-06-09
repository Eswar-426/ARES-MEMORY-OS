use ares_core::AresError;
use super::builder::GraphBuilder;
use super::super::entities::service::EntityService;
use std::sync::Arc;

pub struct EventProcessor {
    entity_service: Arc<EntityService>,
}

impl EventProcessor {
    pub fn new(entity_service: Arc<EntityService>) -> Self {
        Self { entity_service }
    }

    pub async fn process_event(&self, event_type: &str, payload: &serde_json::Value) -> Result<(), AresError> {
        if let Some(entity) = GraphBuilder::build_entity_from_event(event_type, payload) {
            self.entity_service.create_entity(entity).await?;
        }
        Ok(())
    }
}
