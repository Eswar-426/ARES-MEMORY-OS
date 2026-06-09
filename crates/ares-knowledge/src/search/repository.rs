use super::super::entities::models::Entity;
use ares_core::AresError;
use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

pub struct SearchRepository;

impl SearchRepository {
    pub fn new() -> Self {
        Self
    }

    pub fn search_entities(
        &self,
        conn: &Connection,
        query: &str,
    ) -> Result<Vec<Entity>, AresError> {
        let pattern = format!("%{}%", query);
        let mut stmt = conn
            .prepare(
                "SELECT * FROM graph_entities WHERE name LIKE ?1 OR description LIKE ?1 LIMIT 50",
            )
            .map_err(|e| AresError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![pattern], |row| {
                let id_str: String = row.get("id").unwrap();
                let props_str: String = row.get("properties").unwrap_or_else(|_| "{}".to_string());
                let created_at_str: String = row
                    .get("created_at")
                    .unwrap_or_else(|_| Utc::now().to_rfc3339());
                let updated_at_str: String = row
                    .get("updated_at")
                    .unwrap_or_else(|_| Utc::now().to_rfc3339());

                Ok(Entity {
                    id: Uuid::parse_str(&id_str).unwrap(),
                    entity_type: row.get("entity_type").unwrap(),
                    name: row.get("name").unwrap(),
                    description: row.get("description").unwrap_or(None),
                    properties: serde_json::from_str(&props_str).unwrap_or(serde_json::Value::Null),
                    embedding: None,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                        .unwrap()
                        .into(),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                        .unwrap()
                        .into(),
                    valid_from: None,
                    valid_to: None,
                    confidence_score: row.get("confidence_score").unwrap_or(1.0),
                    source_event_id: None,
                })
            })
            .map_err(|e| AresError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| AresError::Database(e.to_string()))?);
        }
        Ok(results)
    }
}

impl Default for SearchRepository {
    fn default() -> Self {
        Self::new()
    }
}
