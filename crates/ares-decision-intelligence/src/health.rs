use ares_core::{AresError, ProjectId};
use ares_store::db::Store;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionHealthSnapshot {
    pub id: String,
    pub project_id: ProjectId,
    pub snapshot_time: i64,
    pub total_decisions: usize,
    pub approved_decisions: usize,
    pub decisions_with_evidence: usize,
    pub decisions_with_consequences: usize,
    pub decisions_without_owner: usize,
    pub health_score: f32,
}

pub struct DecisionHealthEngine {
    store: Store,
}

impl DecisionHealthEngine {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn generate_snapshot(&self, project_id: &ProjectId) -> Result<DecisionHealthSnapshot, AresError> {
        let conn = self.store.get_conn()?;
        
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM decision_records").map_err(AresError::db)?;
        let total_decisions: usize = stmt.query_row([], |r| r.get(0)).unwrap_or(0);

        let mut stmt = conn.prepare("SELECT COUNT(*) FROM decision_records WHERE approval_status = '\"approved\"'").map_err(AresError::db)?;
        let approved_decisions: usize = stmt.query_row([], |r| r.get(0)).unwrap_or(0);

        let mut stmt = conn.prepare("SELECT COUNT(*) FROM decision_records WHERE owner IS NULL").map_err(AresError::db)?;
        let decisions_without_owner: usize = stmt.query_row([], |r| r.get(0)).unwrap_or(0);

        let mut stmt = conn.prepare("SELECT COUNT(DISTINCT decision_id) FROM decision_evidence").map_err(AresError::db)?;
        let decisions_with_evidence: usize = stmt.query_row([], |r| r.get(0)).unwrap_or(0);

        let mut stmt = conn.prepare("SELECT COUNT(*) FROM decision_records WHERE consequences != '[]' AND consequences != ''").map_err(AresError::db)?;
        let decisions_with_consequences: usize = stmt.query_row([], |r| r.get(0)).unwrap_or(0);

        let mut health_score = 100.0;
        
        if total_decisions > 0 {
            // Deduct points for unowned decisions
            health_score -= (decisions_without_owner as f32 / total_decisions as f32) * 30.0;
            // Deduct points for missing evidence
            let missing_evidence = total_decisions.saturating_sub(decisions_with_evidence);
            health_score -= (missing_evidence as f32 / total_decisions as f32) * 30.0;
            // Deduct points for missing consequences
            let missing_consequences = total_decisions.saturating_sub(decisions_with_consequences);
            health_score -= (missing_consequences as f32 / total_decisions as f32) * 40.0;
        }

        health_score = health_score.max(0.0);

        let snapshot = DecisionHealthSnapshot {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: project_id.clone(),
            snapshot_time: Utc::now().timestamp_micros(),
            total_decisions,
            approved_decisions,
            decisions_with_evidence,
            decisions_with_consequences,
            decisions_without_owner,
            health_score,
        };

        conn.execute(
            "INSERT INTO decision_health_snapshots (
                id, project_id, snapshot_time, total_decisions, 
                approved_decisions, decisions_with_evidence, 
                decisions_with_consequences, decisions_without_owner, health_score
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                snapshot.id,
                snapshot.project_id.as_str(),
                snapshot.snapshot_time,
                snapshot.total_decisions,
                snapshot.approved_decisions,
                snapshot.decisions_with_evidence,
                snapshot.decisions_with_consequences,
                snapshot.decisions_without_owner,
                snapshot.health_score,
            ],
        ).map_err(AresError::db)?;

        Ok(snapshot)
    }
}
