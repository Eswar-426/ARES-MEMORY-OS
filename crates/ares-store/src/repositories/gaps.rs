use crate::db::Store;
use ares_core::{AresError, ProjectId};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapType {
    CodeWithoutDecision,
    DecisionWithoutCode,
    OrphanedRequirement,
    StaleDecision,
    UnknownOwnership,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapAlert {
    pub gap_type: GapType,
    pub node_id: String,
    pub node_label: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScore {
    pub overall: f64,
    pub base_score: f64,
    pub decision_bonus: f64,
    pub files_with_decisions_term: f64,
    pub decisions_with_requirements_term: f64,
    pub files_with_owners_term: f64,
    pub fresh_decisions_term: f64,

    pub total_files: i64,
    pub files_with_decisions: i64,
    pub total_decisions: i64,
    pub decisions_with_requirements: i64,
    pub files_with_owners: i64,
    pub stale_decisions: i64,
    pub fresh_decisions: i64,
}

pub struct SqliteGapRepository {
    store: Store,
}

impl SqliteGapRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn calculate_health_score(&self, project_id: &ProjectId) -> Result<HealthScore, AresError> {
        let conn = self.store.get_conn()?;

        let total_files: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM graph_nodes WHERE project_id = ?1 AND node_type = 'file'",
                params![project_id.as_str()],
                |row| row.get(0),
            ).map_err(|e| AresError::Database(e.to_string()))?;

        let files_with_decisions: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT n.id) FROM graph_nodes n 
             JOIN graph_edges e ON e.to_node_id = n.id
             JOIN graph_nodes d ON e.from_node_id = d.id 
             WHERE n.project_id = ?1 AND n.node_type = 'file' AND d.node_type = 'decision'",
                params![project_id.as_str()],
                |row| row.get(0),
            ).map_err(|e| AresError::Database(e.to_string()))?;

        let total_decisions: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM graph_nodes WHERE project_id = ?1 AND node_type = 'decision'",
                params![project_id.as_str()],
                |row| row.get(0),
            ).map_err(|e| AresError::Database(e.to_string()))?;

        let decisions_with_requirements: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT d.id) FROM graph_nodes d
             JOIN graph_edges e ON e.from_node_id = d.id OR e.to_node_id = d.id
             JOIN graph_nodes r ON (e.to_node_id = r.id AND r.node_type = 'requirement') OR (e.from_node_id = r.id AND r.node_type = 'requirement')
             WHERE d.project_id = ?1 AND d.node_type = 'decision'",
            params![project_id.as_str()],
            |row| row.get(0),
        ).map_err(|e| AresError::Database(e.to_string()))?;

        let files_with_owners: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT n.id) FROM graph_nodes n
             JOIN graph_edges e ON e.to_node_id = n.id
             JOIN graph_nodes p ON e.from_node_id = p.id
             WHERE n.project_id = ?1 AND n.node_type = 'file' AND p.node_type IN ('person', 'team') AND e.edge_type IN ('authored_by', 'contributed_to')",
            params![project_id.as_str()],
            |row| row.get(0),
        ).map_err(|e| AresError::Database(e.to_string()))?;

        let thirty_days_micros = 30 * 24 * 60 * 60 * 1_000_000_i64;
        let threshold = Self::now_micros() - thirty_days_micros;

        let fresh_decisions: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM graph_nodes 
             WHERE project_id = ?1 AND node_type = 'decision' AND created_at > ?2",
                params![project_id.as_str(), threshold],
                |row| row.get(0),
            ).map_err(|e| AresError::Database(e.to_string()))?;

        let stale_decisions = total_decisions - fresh_decisions;

        // --- Health Score Calculation ---
        // Base score: file ownership (always measurable via git blame)
        let base_score = if total_files == 0 {
            100.0
        } else {
            (files_with_owners as f64 / (total_files as f64)) * 100.0
        };

        // Decision health bonus: 0-20 points, only calculated when decisions exist
        let decision_bonus = if total_decisions == 0 {
            0.0
        } else {
            let coverage = ((files_with_decisions as f64) / (total_files as f64)).min(1.0);
            let quality = if total_decisions == 0 {
                0.0
            } else {
                ((decisions_with_requirements as f64) / (total_decisions as f64)).min(1.0)
            };
            let freshness = if stale_decisions == 0 {
                if fresh_decisions > 0 { 1.0 } else { 0.5 }
            } else {
                ((fresh_decisions as f64) / (stale_decisions as f64)).min(1.0)
            };
            // Weighted: coverage 30%, quality 30%, freshness 40%, max 20 points
            (coverage * 0.30 + quality * 0.30 + freshness * 0.40) * 20.0
        };

        let overall = (base_score + decision_bonus).min(100.0);

        Ok(HealthScore {
            overall,
            base_score,
            decision_bonus,
            files_with_decisions_term: 0.0,
            decisions_with_requirements_term: 0.0,
            files_with_owners_term: 0.0,
            fresh_decisions_term: 0.0,
            total_files,
            files_with_decisions,
            total_decisions,
            decisions_with_requirements,
            files_with_owners,
            stale_decisions,
            fresh_decisions,
        })
    }

    fn now_micros() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64
    }

    pub fn get_code_without_decision(
        &self,
        project_id: &ProjectId,
        older_than_days: i64,
    ) -> Result<Vec<GapAlert>, AresError> {
        let threshold = Self::now_micros() - (older_than_days * 24 * 60 * 60 * 1_000_000);
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, label, file_path 
             FROM graph_nodes 
             WHERE project_id = ?1 
               AND node_type = 'file' 
               AND created_at < ?2
               AND deleted_at IS NULL
               AND NOT EXISTS (
                   SELECT 1 FROM graph_edges e
                   JOIN graph_nodes dn ON (e.from_node_id = dn.id OR e.to_node_id = dn.id)
                   WHERE (e.from_node_id = graph_nodes.id OR e.to_node_id = graph_nodes.id)
                     AND dn.node_type = 'decision'
                     AND e.valid_until IS NULL
                     AND dn.deleted_at IS NULL
               )",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![project_id.as_str(), threshold], |row| {
                let id: String = row.get(0)?;
                let label: String = row.get(1)?;
                let path: Option<String> = row.get(2)?;
                let path_str = path.unwrap_or_else(|| label.clone());
                Ok(GapAlert {
                    gap_type: GapType::CodeWithoutDecision,
                    node_id: id,
                    node_label: label,
                    details: format!("{} has no recorded decision", path_str),
                })
            })
            .map_err(AresError::db)?;

        let mut alerts = Vec::new();
        for r in rows {
            alerts.push(r.map_err(AresError::db)?);
        }
        Ok(alerts)
    }

    pub fn get_decisions_without_code(
        &self,
        project_id: &ProjectId,
        older_than_days: i64,
    ) -> Result<Vec<GapAlert>, AresError> {
        let threshold = Self::now_micros() - (older_than_days * 24 * 60 * 60 * 1_000_000);
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, label 
             FROM graph_nodes 
             WHERE project_id = ?1 
               AND node_type = 'decision' 
               AND created_at < ?2
               AND deleted_at IS NULL
               AND NOT EXISTS (
                   SELECT 1 FROM graph_edges e
                   JOIN graph_nodes fn ON (e.from_node_id = fn.id OR e.to_node_id = fn.id)
                   WHERE (e.from_node_id = graph_nodes.id OR e.to_node_id = graph_nodes.id)
                     AND fn.node_type = 'file'
                     AND e.valid_until IS NULL
                     AND fn.deleted_at IS NULL
               )",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![project_id.as_str(), threshold], |row| {
                let id: String = row.get(0)?;
                let label: String = row.get(1)?;
                Ok(GapAlert {
                    gap_type: GapType::DecisionWithoutCode,
                    node_id: id.clone(),
                    node_label: label.clone(),
                    details: format!("Decision {} ({}) has no linked implementation", id, label),
                })
            })
            .map_err(AresError::db)?;

        let mut alerts = Vec::new();
        for r in rows {
            alerts.push(r.map_err(AresError::db)?);
        }
        Ok(alerts)
    }

    pub fn get_orphaned_requirements(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<GapAlert>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, label 
             FROM graph_nodes 
             WHERE project_id = ?1 
               AND node_type = 'requirement'
               AND deleted_at IS NULL
               AND NOT EXISTS (
                   SELECT 1 FROM graph_edges e
                   JOIN graph_nodes dn ON (e.from_node_id = dn.id OR e.to_node_id = dn.id)
                   WHERE (e.from_node_id = graph_nodes.id OR e.to_node_id = graph_nodes.id)
                     AND dn.node_type = 'decision'
                     AND e.valid_until IS NULL
                     AND dn.deleted_at IS NULL
               )",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![project_id.as_str()], |row| {
                let id: String = row.get(0)?;
                let label: String = row.get(1)?;
                Ok(GapAlert {
                    gap_type: GapType::OrphanedRequirement,
                    node_id: id.clone(),
                    node_label: label.clone(),
                    details: format!("REQ-{} ({}) has no implementing decisions", id, label),
                })
            })
            .map_err(AresError::db)?;

        let mut alerts = Vec::new();
        for r in rows {
            alerts.push(r.map_err(AresError::db)?);
        }
        Ok(alerts)
    }

    pub fn get_stale_decisions(
        &self,
        project_id: &ProjectId,
        stale_threshold_days: i64,
    ) -> Result<Vec<GapAlert>, AresError> {
        let threshold_us = stale_threshold_days * 24 * 60 * 60 * 1_000_000;
        let conn = self.store.get_conn()?;

        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT d.id, d.label, f.file_path, f.label
             FROM graph_nodes f
             JOIN graph_edges e ON (e.from_node_id = f.id OR e.to_node_id = f.id)
             JOIN graph_nodes d ON (e.from_node_id = d.id OR e.to_node_id = d.id)
             WHERE f.project_id = ?1 
               AND f.node_type = 'file'
               AND d.node_type = 'decision'
               AND f.deleted_at IS NULL
               AND d.deleted_at IS NULL
               AND e.valid_until IS NULL
               AND f.updated_at > (d.created_at + ?2)
               AND NOT EXISTS (
                   SELECT 1 FROM graph_edges e2
                   JOIN graph_nodes d2 ON (e2.from_node_id = d2.id OR e2.to_node_id = d2.id)
                   WHERE (e2.from_node_id = f.id OR e2.to_node_id = f.id)
                     AND d2.node_type = 'decision'
                     AND e2.valid_until IS NULL
                     AND d2.deleted_at IS NULL
                     AND d2.created_at > d.created_at
               )",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![project_id.as_str(), threshold_us], |row| {
                let id: String = row.get(0)?;
                let label: String = row.get(1)?;
                let path: Option<String> = row.get(2)?;
                let f_label: String = row.get(3)?;
                let path_str = path.unwrap_or(f_label);
                Ok(GapAlert {
                    gap_type: GapType::StaleDecision,
                    node_id: id.clone(),
                    node_label: label,
                    details: format!(
                        "Decision {} may be stale. {} changed significantly",
                        id, path_str
                    ),
                })
            })
            .map_err(AresError::db)?;

        let mut alerts = Vec::new();
        for r in rows {
            alerts.push(r.map_err(AresError::db)?);
        }
        Ok(alerts)
    }

    pub fn get_unknown_ownership(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<GapAlert>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, label, file_path 
             FROM graph_nodes 
             WHERE project_id = ?1 
               AND node_type = 'file'
               AND deleted_at IS NULL
               AND NOT EXISTS (
                   SELECT 1 FROM graph_edges e
                   WHERE (e.from_node_id = graph_nodes.id OR e.to_node_id = graph_nodes.id)
                     AND e.edge_type IN ('authored_by', 'contributed_to', 'owns')
                     AND e.valid_until IS NULL
               )",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![project_id.as_str()], |row| {
                let id: String = row.get(0)?;
                let label: String = row.get(1)?;
                let path: Option<String> = row.get(2)?;
                let path_str = path.unwrap_or_else(|| label.clone());
                Ok(GapAlert {
                    gap_type: GapType::UnknownOwnership,
                    node_id: id,
                    node_label: label,
                    details: format!("{} has no clear owner", path_str),
                })
            })
            .map_err(AresError::db)?;

        let mut alerts = Vec::new();
        for r in rows {
            alerts.push(r.map_err(AresError::db)?);
        }
        Ok(alerts)
    }
}
