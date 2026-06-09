use ares_core::AresError;
use rusqlite::{params, Connection};
use uuid::Uuid;
use super::models::Entity;

pub struct EntityRepository;

impl EntityRepository {
    pub fn new() -> Self {
        Self
    }

    pub fn insert(&self, conn: &Connection, entity: &Entity) -> Result<(), AresError> {
        let props_json = serde_json::to_string(&entity.properties).unwrap_or_else(|_| "{}".to_string());
        
        let source_event_id = entity.source_event_id.map(|u| u.to_string());
        let valid_from = entity.valid_from.map(|dt| dt.to_rfc3339());
        let valid_to = entity.valid_to.map(|dt| dt.to_rfc3339());
        
        conn.execute(
            "INSERT INTO graph_entities (
                id, entity_type, name, description, properties,
                created_at, updated_at, valid_from, valid_to,
                confidence_score, source_event_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                entity.id.to_string(),
                entity.entity_type,
                entity.name,
                entity.description,
                props_json,
                entity.created_at.to_rfc3339(),
                entity.updated_at.to_rfc3339(),
                valid_from,
                valid_to,
                entity.confidence_score,
                source_event_id,
            ],
        ).map_err(|e| AresError::Database(e.to_string()))?;
        
        Ok(())
    }

    pub fn get_by_id(&self, conn: &Connection, id: Uuid) -> Result<Option<Entity>, AresError> {
        let mut stmt = conn.prepare("SELECT * FROM graph_entities WHERE id = ?1")
            .map_err(|e| AresError::Database(e.to_string()))?;
            
        let mut rows = stmt.query(params![id.to_string()])
            .map_err(|e| AresError::Database(e.to_string()))?;

        if let Some(row) = rows.next().map_err(|e| AresError::Database(e.to_string()))? {
            let id_str: String = row.get("id").unwrap();
            let props_str: String = row.get("properties").unwrap_or_else(|_| "{}".to_string());
            let created_at_str: String = row.get("created_at").unwrap();
            let updated_at_str: String = row.get("updated_at").unwrap();
            
            let entity = Entity {
                id: Uuid::parse_str(&id_str).unwrap(),
                entity_type: row.get("entity_type").unwrap(),
                name: row.get("name").unwrap(),
                description: row.get("description").ok(),
                properties: serde_json::from_str(&props_str).unwrap_or(serde_json::Value::Null),
                embedding: None, // Simplified for Phase A scaffolding
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str).unwrap().into(),
                updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str).unwrap().into(),
                valid_from: None,
                valid_to: None,
                confidence_score: row.get("confidence_score").unwrap_or(1.0),
                source_event_id: None,
            };
            Ok(Some(entity))
        } else {
            Ok(None)
        }
    }

    pub fn update(&self, conn: &Connection, entity: &Entity) -> Result<(), AresError> {
        let props_json = serde_json::to_string(&entity.properties).unwrap_or_else(|_| "{}".to_string());
        
        conn.execute(
            "UPDATE graph_entities SET 
                name = ?1, description = ?2, properties = ?3, updated_at = ?4, confidence_score = ?5
             WHERE id = ?6",
            params![
                entity.name,
                entity.description,
                props_json,
                entity.updated_at.to_rfc3339(),
                entity.confidence_score,
                entity.id.to_string(),
            ],
        ).map_err(|e| AresError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn delete(&self, conn: &Connection, id: Uuid) -> Result<(), AresError> {
        conn.execute(
            "DELETE FROM graph_entities WHERE id = ?1",
            params![id.to_string()],
        ).map_err(|e| AresError::Database(e.to_string()))?;
        Ok(())
    }
}

impl Default for EntityRepository {
    fn default() -> Self {
        Self::new()
    }
}
