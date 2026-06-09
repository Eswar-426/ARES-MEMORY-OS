use ares_core::AresError;
use rusqlite::Connection;
// use uuid::Uuid;

pub struct GraphRepository;

impl GraphRepository {
    pub fn new() -> Self {
        Self
    }

    pub fn get_statistics(&self, conn: &Connection) -> Result<serde_json::Value, AresError> {
        let entity_count: i64 = conn
            .query_row("SELECT count(*) FROM graph_entities", [], |row| row.get(0))
            .map_err(|e| AresError::Database(e.to_string()))?;
        let relationship_count: i64 = conn
            .query_row("SELECT count(*) FROM graph_relationships", [], |row| {
                row.get(0)
            })
            .map_err(|e| AresError::Database(e.to_string()))?;

        Ok(serde_json::json!({
            "entities": entity_count,
            "relationships": relationship_count,
        }))
    }
}

impl Default for GraphRepository {
    fn default() -> Self {
        Self::new()
    }
}
