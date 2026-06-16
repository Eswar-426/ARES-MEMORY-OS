use crate::models::Requirement;
use ares_core::{AresError, RequirementId, RequirementRevisionId};
use ares_store::db::Store;
use chrono::Utc;
use rusqlite::params;
use tracing::debug;

pub struct RequirementRevision {
    pub id: RequirementRevisionId,
    pub requirement_id: RequirementId,
    pub revision_number: u32,
    pub previous_state: serde_json::Value,
    pub new_state: serde_json::Value,
    pub changed_fields: Vec<String>,
    pub changed_by: Option<String>,
    pub change_reason: Option<String>,
    pub created_at: i64,
}

pub struct RequirementHistory {
    store: Store,
}

impl RequirementHistory {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn record_revision(
        &self,
        requirement_id: &RequirementId,
        previous: &Requirement,
        updated: &Requirement,
        changed_by: Option<&str>,
        reason: Option<&str>,
    ) -> Result<RequirementRevision, AresError> {
        let conn = self.store.get_conn()?;
        
        let current_rev = self.current_revision_number(requirement_id)?;
        let next_rev = current_rev + 1;
        let rev_id = RequirementRevisionId::new();
        let now = Utc::now().timestamp_micros();

        let prev_json = serde_json::to_value(previous).map_err(|e| AresError::validation(e.to_string()))?;
        let new_json = serde_json::to_value(updated).map_err(|e| AresError::validation(e.to_string()))?;
        
        // Compute changed fields
        let mut changed = Vec::new();
        if previous.title != updated.title { changed.push("title".to_string()); }
        if previous.description != updated.description { changed.push("description".to_string()); }
        if previous.requirement_type != updated.requirement_type { changed.push("requirement_type".to_string()); }
        if previous.status != updated.status { changed.push("status".to_string()); }
        if previous.priority != updated.priority { changed.push("priority".to_string()); }
        if previous.owner != updated.owner { changed.push("owner".to_string()); }
        if previous.tags != updated.tags { changed.push("tags".to_string()); }

        let changed_fields_json = serde_json::to_string(&changed).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "INSERT INTO requirement_revisions (
                id, requirement_id, revision_number, previous_state, new_state,
                changed_fields, changed_by, change_reason, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                rev_id.as_str(),
                requirement_id.as_str(),
                next_rev,
                serde_json::to_string(&prev_json).unwrap(),
                serde_json::to_string(&new_json).unwrap(),
                changed_fields_json,
                changed_by,
                reason,
                now,
            ],
        )
        .map_err(AresError::db)?;

        debug!(req_id = %requirement_id, rev = next_rev, "Requirement revision recorded");

        Ok(RequirementRevision {
            id: rev_id,
            requirement_id: requirement_id.clone(),
            revision_number: next_rev,
            previous_state: prev_json,
            new_state: new_json,
            changed_fields: changed,
            changed_by: changed_by.map(|s| s.to_string()),
            change_reason: reason.map(|s| s.to_string()),
            created_at: now,
        })
    }

    pub fn get_history(
        &self,
        requirement_id: &RequirementId,
    ) -> Result<Vec<RequirementRevision>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, requirement_id, revision_number, previous_state, new_state,
                 changed_fields, changed_by, change_reason, created_at
                 FROM requirement_revisions
                 WHERE requirement_id = ?1
                 ORDER BY revision_number ASC"
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![requirement_id.as_str()], row_to_revision)
            .map_err(AresError::db)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)
    }

    pub fn get_at_revision(
        &self,
        requirement_id: &RequirementId,
        revision_number: u32,
    ) -> Result<Option<RequirementRevision>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, requirement_id, revision_number, previous_state, new_state,
                 changed_fields, changed_by, change_reason, created_at
                 FROM requirement_revisions
                 WHERE requirement_id = ?1 AND revision_number = ?2"
            )
            .map_err(AresError::db)?;

        let result = stmt.query_row(params![requirement_id.as_str(), revision_number], row_to_revision);

        match result {
            Ok(rev) => Ok(Some(rev)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AresError::db(e)),
        }
    }

    pub fn current_revision_number(
        &self,
        requirement_id: &RequirementId,
    ) -> Result<u32, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT COALESCE(MAX(revision_number), 0)
                 FROM requirement_revisions
                 WHERE requirement_id = ?1"
            )
            .map_err(AresError::db)?;

        stmt.query_row(params![requirement_id.as_str()], |row| row.get(0))
            .map_err(AresError::db)
    }
}

fn row_to_revision(row: &rusqlite::Row<'_>) -> Result<RequirementRevision, rusqlite::Error> {
    let prev_str: String = row.get(3)?;
    let new_str: String = row.get(4)?;
    let fields_str: String = row.get(5)?;

    Ok(RequirementRevision {
        id: RequirementRevisionId::from(row.get::<_, String>(0)?),
        requirement_id: RequirementId::from(row.get::<_, String>(1)?),
        revision_number: row.get(2)?,
        previous_state: serde_json::from_str(&prev_str).unwrap_or(serde_json::Value::Null),
        new_state: serde_json::from_str(&new_str).unwrap_or(serde_json::Value::Null),
        changed_fields: serde_json::from_str(&fields_str).unwrap_or_default(),
        changed_by: row.get(6)?,
        change_reason: row.get(7)?,
        created_at: row.get(8)?,
    })
}
