use super::super::entities::models::Entity;
use uuid::Uuid;
use chrono::Utc;

pub struct GraphBuilder;

impl GraphBuilder {
    pub fn build_entity_from_event(event_type: &str, payload: &serde_json::Value) -> Option<Entity> {
        // Mock implementation
        Some(Entity {
            id: Uuid::now_v7(),
            entity_type: event_type.replace("Created", "").replace("Registered", "").to_uppercase(),
            name: "Generated Entity".to_string(),
            description: None,
            properties: payload.clone(),
            embedding: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            valid_from: None,
            valid_to: None,
            confidence_score: 1.0,
            source_event_id: None,
        })
    }
}
