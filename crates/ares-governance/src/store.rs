use crate::models::{ApprovalStatus, ComplianceViolation, GovernanceApprovalRequest};
use ares_core::AresError;
use ares_store::Store;
use rusqlite::params;

pub trait ApprovalStore: Send + Sync {
    fn create_request(&self, request: &GovernanceApprovalRequest) -> Result<(), AresError>;
    fn get_request(&self, id: &str) -> Result<Option<GovernanceApprovalRequest>, AresError>;
    fn update_status(
        &self,
        id: &str,
        status: ApprovalStatus,
        approved_by: Option<&str>,
    ) -> Result<(), AresError>;
    fn list_requests_by_project(
        &self,
        project_id: &str,
        status: Option<ApprovalStatus>,
    ) -> Result<Vec<GovernanceApprovalRequest>, AresError>;
}

pub struct SqliteApprovalStore {
    store: Store,
}

impl SqliteApprovalStore {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

impl ApprovalStore for SqliteApprovalStore {
    fn create_request(&self, request: &GovernanceApprovalRequest) -> Result<(), AresError> {
        let violations_json = serde_json::to_string(&request.violations)
            .map_err(|e| AresError::Database(format!("Failed to serialize violations: {}", e)))?;

        let status_str = match request.status {
            ApprovalStatus::Pending => "Pending",
            ApprovalStatus::Approved => "Approved",
            ApprovalStatus::Rejected => "Rejected",
            ApprovalStatus::Expired => "Expired",
        };

        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO governance_approval_requests (
                id, project_id, workflow_id, status, requested_by, approved_by,
                created_at, updated_at, expires_at, violations_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                request.id,
                request.project_id,
                request.workflow_id,
                status_str,
                request.requested_by,
                request.approved_by,
                request.requested_at,
                request.updated_at,
                request.expires_at,
                violations_json,
            ],
        )
        .map_err(AresError::db)?;

        Ok(())
    }

    fn get_request(&self, id: &str) -> Result<Option<GovernanceApprovalRequest>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, workflow_id, status, requested_by, approved_by,
                created_at, updated_at, expires_at, violations_json
             FROM governance_approval_requests WHERE id = ?1",
            )
            .map_err(AresError::db)?;

        let result = stmt.query_row(params![id], |r| {
            let status_str: String = r.get(3)?;
            let status = match status_str.as_str() {
                "Approved" => ApprovalStatus::Approved,
                "Rejected" => ApprovalStatus::Rejected,
                "Expired" => ApprovalStatus::Expired,
                _ => ApprovalStatus::Pending,
            };

            let violations_str: String = r.get(9)?;
            let violations: Vec<ComplianceViolation> =
                serde_json::from_str(&violations_str).unwrap_or_default();

            Ok(GovernanceApprovalRequest {
                id: r.get(0)?,
                project_id: r.get(1)?,
                workflow_id: r.get(2)?,
                status,
                requested_by: r.get(4)?,
                approved_by: r.get(5)?,
                requested_at: r.get(6)?,
                updated_at: r.get(7)?,
                expires_at: r.get(8)?,
                violations,
            })
        });

        match result {
            Ok(req) => Ok(Some(req)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AresError::db(e)),
        }
    }

    fn update_status(
        &self,
        id: &str,
        status: ApprovalStatus,
        approved_by: Option<&str>,
    ) -> Result<(), AresError> {
        let status_str = match status {
            ApprovalStatus::Pending => "Pending",
            ApprovalStatus::Approved => "Approved",
            ApprovalStatus::Rejected => "Rejected",
            ApprovalStatus::Expired => "Expired",
        };

        let now = chrono::Utc::now().timestamp();

        let conn = self.store.get_conn()?;
        conn.execute(
            "UPDATE governance_approval_requests
             SET status = ?1, approved_by = ?2, updated_at = ?3
             WHERE id = ?4",
            params![status_str, approved_by, now, id,],
        )
        .map_err(AresError::db)?;

        Ok(())
    }

    fn list_requests_by_project(
        &self,
        project_id: &str,
        status: Option<ApprovalStatus>,
    ) -> Result<Vec<GovernanceApprovalRequest>, AresError> {
        let conn = self.store.get_conn()?;

        let mut query_str = String::from(
            "SELECT id, project_id, workflow_id, status, requested_by, approved_by,
                created_at, updated_at, expires_at, violations_json
             FROM governance_approval_requests WHERE project_id = ?1",
        );

        let mut status_str_opt = None;
        if let Some(s) = status {
            query_str.push_str(" AND status = ?2");
            status_str_opt = Some(match s {
                ApprovalStatus::Pending => "Pending",
                ApprovalStatus::Approved => "Approved",
                ApprovalStatus::Rejected => "Rejected",
                ApprovalStatus::Expired => "Expired",
            });
        }

        query_str.push_str(" ORDER BY created_at DESC LIMIT 100");

        let mut stmt = conn.prepare(&query_str).map_err(AresError::db)?;

        let row_mapper = |r: &rusqlite::Row| {
            let status_str: String = r.get(3)?;
            let s = match status_str.as_str() {
                "Approved" => ApprovalStatus::Approved,
                "Rejected" => ApprovalStatus::Rejected,
                "Expired" => ApprovalStatus::Expired,
                _ => ApprovalStatus::Pending,
            };

            let violations_str: String = r.get(9)?;
            let violations: Vec<ComplianceViolation> =
                serde_json::from_str(&violations_str).unwrap_or_default();

            Ok(GovernanceApprovalRequest {
                id: r.get(0)?,
                project_id: r.get(1)?,
                workflow_id: r.get(2)?,
                status: s,
                requested_by: r.get(4)?,
                approved_by: r.get(5)?,
                requested_at: r.get(6)?,
                updated_at: r.get(7)?,
                expires_at: r.get(8)?,
                violations,
            })
        };

        let rows = if let Some(s) = status_str_opt {
            stmt.query_map(params![project_id, s], row_mapper)
                .map_err(AresError::db)?
        } else {
            stmt.query_map(params![project_id], row_mapper)
                .map_err(AresError::db)?
        };

        rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)
    }
}
