use async_trait::async_trait;
use rusqlite::params;

use crate::db::Store;
use ares_core::types::drift::{DriftCandidate, DriftType};
use ares_core::AresError;

#[async_trait]
pub trait DriftRepository: Send + Sync {
    async fn record_candidate(&self, candidate: DriftCandidate) -> Result<(), AresError>;
    async fn get_candidates_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<DriftCandidate>, AresError>;
}

pub struct SqliteDriftRepository {
    store: Store,
}

impl SqliteDriftRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

#[async_trait]
impl DriftRepository for SqliteDriftRepository {
    async fn record_candidate(&self, candidate: DriftCandidate) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let evidence_ids_json = serde_json::to_string(&candidate.evidence_ids)
            .map_err(|e| AresError::Serialization(e.to_string()))?;

        conn.execute(
            "INSERT INTO drift_candidates (id, project_id, target_node_id, drift_type, confidence, evidence_ids, rationale, detected_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                candidate.id,
                candidate.project_id,
                candidate.target_node_id,
                serde_json::to_string(&candidate.drift_type).unwrap_or_default().trim_matches('"').to_string(),
                candidate.confidence,
                evidence_ids_json,
                candidate.rationale,
                candidate.detected_at.timestamp(),
            ],
        )
        .map_err(AresError::db)?;

        Ok(())
    }

    async fn get_candidates_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<DriftCandidate>, AresError> {
        let conn = self.store.get_conn()?;

        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, target_node_id, drift_type, confidence, evidence_ids, rationale, detected_at
                 FROM drift_candidates
                 WHERE project_id = ?1
                 ORDER BY detected_at DESC"
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![project_id], |row| {
                let id: String = row.get(0)?;
                let proj_id: String = row.get(1)?;
                let target_node_id: String = row.get(2)?;
                let drift_type_str: String = row.get(3)?;
                let confidence: f32 = row.get(4)?;
                let evidence_ids_json: String = row.get(5)?;
                let rationale: String = row.get(6)?;
                let detected_at_ts: i64 = row.get(7)?;

                // For simplicity we use serde_json to parse the enum or fallback
                let drift_type = serde_json::from_str(&format!("\"{}\"", drift_type_str))
                    .unwrap_or(DriftType::TraceabilityGap); // Default fallback

                let evidence_ids: Vec<String> =
                    serde_json::from_str(&evidence_ids_json).unwrap_or_default();

                use chrono::{TimeZone, Utc};
                let detected_at = Utc.timestamp_opt(detected_at_ts, 0).unwrap();

                Ok(DriftCandidate {
                    id,
                    project_id: proj_id,
                    target_node_id,
                    drift_type,
                    confidence,
                    evidence_ids,
                    rationale,
                    detected_at,
                })
            })
            .map_err(AresError::db)?;

        let mut candidates = Vec::new();
        for r in rows {
            candidates.push(r.map_err(AresError::db)?);
        }

        Ok(candidates)
    }
}
