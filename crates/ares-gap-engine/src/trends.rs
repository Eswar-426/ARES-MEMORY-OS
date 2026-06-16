use crate::models::RepositoryHealthSnapshot;
use ares_core::AresError;
use ares_store::Store;
use std::sync::Arc;

pub struct HealthTrendEngine {
    store: Arc<Store>,
}

impl HealthTrendEngine {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    /// Saves a health snapshot into historical trends.
    pub fn save_snapshot(&self, snapshot: &RepositoryHealthSnapshot) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        
        let component_scores_json = serde_json::to_string(&snapshot.component_scores)
            .map_err(|e| AresError::validation(format!("Serialization failed: {}", e)))?;

        conn.execute(
            "INSERT INTO repository_health_trends (id, project_id, snapshot_time, overall_score, component_scores, total_gaps, critical_gaps)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                &snapshot.snapshot_id,
                snapshot.project_id.as_str(),
                snapshot.snapshot_time,
                snapshot.overall_score,
                &component_scores_json,
                snapshot.total_gaps,
                snapshot.critical_gaps,
            ),
        ).map_err(AresError::db)?;

        Ok(())
    }

    /// Retrieves the historical trends for a project.
    pub fn get_trends(&self, project_id: &ares_core::id::ProjectId) -> Result<Vec<RepositoryHealthSnapshot>, AresError> {
        let conn = self.store.get_conn()?;
        
        let mut stmt = conn.prepare(
            "SELECT id, snapshot_time, overall_score, component_scores, total_gaps, critical_gaps 
             FROM repository_health_trends 
             WHERE project_id = ?1 
             ORDER BY snapshot_time ASC"
        ).map_err(AresError::db)?;

        let trends = stmt.query_map([project_id.as_str()], |row| {
            let component_scores_json: String = row.get(3)?;
            let component_scores = serde_json::from_str(&component_scores_json).unwrap_or_default();

            Ok(RepositoryHealthSnapshot {
                snapshot_id: row.get(0)?,
                project_id: project_id.clone(),
                snapshot_time: row.get(1)?,
                overall_score: row.get(2)?,
                component_scores,
                total_gaps: row.get(4)?,
                critical_gaps: row.get(5)?,
            })
        }).map_err(AresError::db)?
        .collect::<Result<Vec<_>, _>>().map_err(AresError::db)?;

        Ok(trends)
    }
}
