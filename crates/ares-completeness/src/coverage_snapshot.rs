use crate::models::{CoverageMetrics, CoverageSnapshot};
use ares_core::AresError;
use ares_store::Store;
use chrono::{TimeZone, Utc};
use rusqlite::params;

pub struct CoverageSnapshotRepository {
    store: Store,
}

impl CoverageSnapshotRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn setup_schema(&self) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS coverage_snapshots (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                requirement_coverage REAL NOT NULL,
                decision_coverage REAL NOT NULL,
                architecture_coverage REAL NOT NULL,
                code_coverage REAL NOT NULL,
                test_coverage REAL NOT NULL,
                runtime_coverage REAL NOT NULL,
                overall_coverage REAL NOT NULL
            )",
            [],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    pub fn record_snapshot(&self, snapshot: &CoverageSnapshot) -> Result<(), AresError> {
        self.setup_schema()?; // ensure schema exists
        let conn = self.store.get_conn()?;

        conn.execute(
            "INSERT INTO coverage_snapshots (
                id, project_id, timestamp, 
                requirement_coverage, decision_coverage, architecture_coverage, 
                code_coverage, test_coverage, runtime_coverage, overall_coverage
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                snapshot.id,
                snapshot.project_id,
                snapshot.timestamp.timestamp(),
                snapshot.metrics.requirement_coverage,
                snapshot.metrics.decision_coverage,
                snapshot.metrics.architecture_coverage,
                snapshot.metrics.code_coverage,
                snapshot.metrics.test_coverage,
                snapshot.metrics.runtime_coverage,
                snapshot.overall_coverage,
            ],
        )
        .map_err(AresError::db)?;

        Ok(())
    }

    pub fn get_snapshots_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<CoverageSnapshot>, AresError> {
        self.setup_schema()?;
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare("SELECT id, project_id, timestamp, requirement_coverage, decision_coverage, architecture_coverage, code_coverage, test_coverage, runtime_coverage, overall_coverage FROM coverage_snapshots WHERE project_id = ?1 ORDER BY timestamp ASC")
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map([project_id], |row| {
                let id: String = row.get(0)?;
                let pid: String = row.get(1)?;
                let ts: i64 = row.get(2)?;
                let req_cov: f32 = row.get(3)?;
                let dec_cov: f32 = row.get(4)?;
                let arch_cov: f32 = row.get(5)?;
                let code_cov: f32 = row.get(6)?;
                let test_cov: f32 = row.get(7)?;
                let run_cov: f32 = row.get(8)?;
                let overall: f32 = row.get(9)?;

                let metrics = CoverageMetrics {
                    requirement_coverage: req_cov,
                    decision_coverage: dec_cov,
                    architecture_coverage: arch_cov,
                    code_coverage: code_cov,
                    test_coverage: test_cov,
                    runtime_coverage: run_cov,
                };

                Ok(CoverageSnapshot {
                    id,
                    project_id: pid,
                    timestamp: Utc.timestamp_opt(ts, 0).unwrap(),
                    metrics,
                    overall_coverage: overall,
                })
            })
            .map_err(AresError::db)?;

        let mut snaps = Vec::new();
        for r in rows {
            snaps.push(r.map_err(AresError::db)?);
        }

        Ok(snaps)
    }
}
