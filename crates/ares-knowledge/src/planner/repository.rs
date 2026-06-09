use super::models::GoalState;
use ares_core::AresError;
use rusqlite::{params, Connection};
use uuid::Uuid;

pub struct GoalStateRepository;

impl GoalStateRepository {
    pub fn new() -> Self {
        Self
    }

    pub fn insert(&self, conn: &Connection, state: &GoalState) -> Result<(), AresError> {
        conn.execute(
            "INSERT INTO goal_states (id, entity_id, status, progress, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                state.id.to_string(),
                state.entity_id.to_string(),
                state.status,
                state.progress,
                state.updated_at.to_rfc3339(),
            ],
        )
        .map_err(|e| AresError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn update(&self, conn: &Connection, state: &GoalState) -> Result<(), AresError> {
        conn.execute(
            "UPDATE goal_states SET status = ?1, progress = ?2, updated_at = ?3 WHERE id = ?4",
            params![
                state.status,
                state.progress,
                state.updated_at.to_rfc3339(),
                state.id.to_string(),
            ],
        )
        .map_err(|e| AresError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn get_by_entity_id(
        &self,
        conn: &Connection,
        entity_id: Uuid,
    ) -> Result<Option<GoalState>, AresError> {
        let mut stmt = conn
            .prepare("SELECT * FROM goal_states WHERE entity_id = ?1")
            .map_err(|e| AresError::Database(e.to_string()))?;

        let mut rows = stmt
            .query(params![entity_id.to_string()])
            .map_err(|e| AresError::Database(e.to_string()))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| AresError::Database(e.to_string()))?
        {
            let id_str: String = row.get("id").unwrap();
            let entity_id_str: String = row.get("entity_id").unwrap();
            let updated_at_str: String = row.get("updated_at").unwrap();

            Ok(Some(GoalState {
                id: Uuid::parse_str(&id_str).unwrap(),
                entity_id: Uuid::parse_str(&entity_id_str).unwrap(),
                status: row.get("status").unwrap(),
                progress: row.get("progress").unwrap(),
                updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                    .unwrap()
                    .into(),
            }))
        } else {
            Ok(None)
        }
    }
}

impl Default for GoalStateRepository {
    fn default() -> Self {
        Self::new()
    }
}
