use crate::models::{ApprovalStatus, ComplianceViolation, GovernanceApprovalRequest};
use crate::store::{ApprovalStore, SqliteApprovalStore};
use ares_core::AresError;
use ares_store::Store;
use std::sync::Arc;

#[derive(Clone)]
pub struct ApprovalEngine {
    store: Arc<Store>,
}

impl ApprovalEngine {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    fn approval_store(&self) -> SqliteApprovalStore {
        SqliteApprovalStore::new((*self.store).clone())
    }

    pub async fn create_request(
        &self,
        project_id: &str,
        workflow_id: &str,
        violations: Vec<ComplianceViolation>,
    ) -> Result<GovernanceApprovalRequest, AresError> {
        let request = GovernanceApprovalRequest {
            id: uuid::Uuid::new_v4().to_string(),
            workflow_id: workflow_id.to_string(),
            project_id: project_id.to_string(),
            violations,
            status: ApprovalStatus::Pending,
            requested_by: "system".to_string(), // In reality, extract from token
            approved_by: None,
            requested_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            expires_at: None,
        };

        let repo = self.approval_store();
        repo.create_request(&request)?;
        Ok(request)
    }

    pub async fn get_request(
        &self,
        id: &str,
    ) -> Result<Option<GovernanceApprovalRequest>, AresError> {
        let repo = self.approval_store();
        repo.get_request(id)
    }

    pub async fn approve_request(
        &self,
        id: &str,
        approved_by: &str,
    ) -> Result<GovernanceApprovalRequest, AresError> {
        let repo = self.approval_store();
        repo.update_status(id, ApprovalStatus::Approved, Some(approved_by))?;

        let req = repo.get_request(id)?;
        if let Some(r) = req {
            Ok(r)
        } else {
            Err(AresError::validation("Request not found after approval"))
        }
    }

    pub async fn reject_request(&self, id: &str) -> Result<(), AresError> {
        let repo = self.approval_store();
        repo.update_status(id, ApprovalStatus::Rejected, None)?;
        Ok(())
    }
}
