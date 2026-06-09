use super::models::Relationship;
use ares_core::AresError;
use rusqlite::{params, Connection};
use uuid::Uuid;

pub struct RelationshipRepository;

impl RelationshipRepository {
    pub fn new() -> Self {
        Self
    }

    pub fn insert(&self, conn: &Connection, rel: &Relationship) -> Result<(), AresError> {
        let props_json =
            serde_json::to_string(&rel.properties).unwrap_or_else(|_| "{}".to_string());

        let source_event_id = rel.source_event_id.map(|u| u.to_string());
        let valid_from = rel.valid_from.map(|dt| dt.to_rfc3339());
        let valid_to = rel.valid_to.map(|dt| dt.to_rfc3339());

        conn.execute(
            "INSERT INTO graph_relationships (
                id, source_entity, target_entity, relationship_type, properties,
                created_at, updated_at, valid_from, valid_to,
                confidence_score, evidence_count, source_event_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                rel.id.to_string(),
                rel.source_entity.to_string(),
                rel.target_entity.to_string(),
                rel.relationship_type,
                props_json,
                rel.created_at.to_rfc3339(),
                rel.updated_at.to_rfc3339(),
                valid_from,
                valid_to,
                rel.confidence_score,
                rel.evidence_count,
                source_event_id,
            ],
        )
        .map_err(|e| AresError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn delete(&self, conn: &Connection, id: Uuid) -> Result<(), AresError> {
        conn.execute(
            "DELETE FROM graph_relationships WHERE id = ?1",
            params![id.to_string()],
        )
        .map_err(|e| AresError::Database(e.to_string()))?;
        Ok(())
    }
}

impl Default for RelationshipRepository {
    fn default() -> Self {
        Self::new()
    }
}
