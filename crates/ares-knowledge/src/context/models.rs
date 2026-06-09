use crate::entities::models::Entity;
use crate::relationships::models::Relationship;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextGraph {
    pub focal_entity_id: Uuid,
    pub entities: Vec<Entity>,
    pub relationships: Vec<Relationship>,
    pub depth: u32,
}
