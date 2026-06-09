use super::super::models::Entity;
use super::matcher::normalize_name;
use ares_core::AresError;
use ares_store::db::Store;
use rusqlite::params;
use uuid::Uuid;

pub struct EntityResolutionService {
    db: Store,
}

impl EntityResolutionService {
    pub fn new(db: Store) -> Self {
        Self { db }
    }

    pub async fn resolve_entity(
        &self,
        name: &str,
        entity_type: &str,
    ) -> Result<Option<Entity>, AresError> {
        let conn = self.db.get_conn()?;

        let normalized = normalize_name(name);

        // Find candidates
        let mut stmt = conn
            .prepare(
                "
            SELECT e.id, e.name, e.entity_type 
            FROM graph_entities e
            LEFT JOIN entity_aliases a ON e.id = a.entity_id
            WHERE e.entity_type = ?1 AND (
                e.name = ?2 OR 
                a.normalized_alias = ?3
            )
        ",
            )
            .map_err(|e| AresError::Database(e.to_string()))?;

        let mut rows = stmt
            .query(params![entity_type, name, normalized])
            .map_err(|e| AresError::Database(e.to_string()))?;

        // For simplicity in Phase A, return first match if any.
        // A real system would load the full Entity via repo.
        if let Some(row) = rows
            .next()
            .map_err(|e| AresError::Database(e.to_string()))?
        {
            let id_str: String = row.get(0).unwrap();
            // Just returning a dummy Entity with the matched ID since we don't have the full repo injected here easily
            // In a real app we'd call EntityService::get_entity
            let entity = Entity {
                id: Uuid::parse_str(&id_str).unwrap(),
                entity_type: row.get(2).unwrap(),
                name: row.get(1).unwrap(),
                description: None,
                properties: serde_json::Value::Null,
                embedding: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                valid_from: None,
                valid_to: None,
                confidence_score: 1.0,
                source_event_id: None,
            };
            return Ok(Some(entity));
        }

        Ok(None)
    }
}
