use serde_json::Value;
use uuid::Uuid;

pub enum KnowledgeCommand {
    CreateEntity {
        entity_type: String,
        name: String,
        description: Option<String>,
        properties: Value,
        source_event_id: Option<Uuid>,
    },
    UpdateEntity {
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        properties: Option<Value>,
    },
    MergeEntity {
        target_id: Uuid,
        source_id: Uuid,
    },
    DeleteEntity {
        id: Uuid,
    },
    CreateRelationship {
        source_id: Uuid,
        target_id: Uuid,
        relationship_type: String,
        properties: Value,
        source_event_id: Option<Uuid>,
    },
    DeleteRelationship {
        id: Uuid,
    },
}
