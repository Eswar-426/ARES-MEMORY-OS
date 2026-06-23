use ares_core::{AresError, DecisionId};
use ares_store::db::Store;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRevision {
    pub id: String,
    pub decision_id: DecisionId,
    pub changed_by: Option<String>,
    pub change_reason: Option<String>,
    pub diff_payload: String,
    pub created_at: i64,
}

pub struct DecisionHistory {
    store: Store,
}

impl DecisionHistory {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn record_revision(
        &self,
        decision_id: &DecisionId,
        changed_by: Option<&str>,
        change_reason: Option<&str>,
        diff_payload: &str,
    ) -> Result<String, AresError> {
        let conn = self.store.get_conn()?;
        let revision_id = format!("dec_rev_{}", uuid::Uuid::new_v4());
        let now = Utc::now().timestamp_micros();

        conn.execute(
            "INSERT INTO decision_revisions (
                id, decision_id, changed_by, change_reason, diff_payload, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                revision_id,
                decision_id.as_str(),
                changed_by,
                change_reason,
                diff_payload,
                now,
            ],
        )
        .map_err(AresError::db)?;

        Ok(revision_id)
    }

    pub fn get_revisions(
        &self,
        decision_id: &DecisionId,
    ) -> Result<Vec<DecisionRevision>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, changed_by, change_reason, diff_payload, created_at 
                 FROM decision_revisions 
                 WHERE decision_id = ?1 
                 ORDER BY created_at DESC",
            )
            .map_err(AresError::db)?;

        let revisions = stmt
            .query_map(params![decision_id.as_str()], |row| {
                Ok(DecisionRevision {
                    id: row.get(0)?,
                    decision_id: decision_id.clone(),
                    changed_by: row.get(1)?,
                    change_reason: row.get(2)?,
                    diff_payload: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(AresError::db)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(AresError::db)?;

        Ok(revisions)
    }
}
