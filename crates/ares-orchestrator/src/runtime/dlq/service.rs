use super::{models::DeadLetterItem, repository::DlqRepository};
use ares_core::AresError;
use chrono::Utc;
use uuid::Uuid;

pub struct DlqService {
    repo: DlqRepository,
}

impl DlqService {
    pub fn new(repo: DlqRepository) -> Self {
        Self { repo }
    }

    pub fn send_to_dlq(
        &self,
        original_queue_id: &str,
        workflow_id: &str,
        execution_key: &str,
        failure_reason: &str,
        attempt_count: i32,
    ) -> Result<DeadLetterItem, AresError> {
        let item = DeadLetterItem {
            id: Uuid::now_v7().to_string(),
            original_queue_id: original_queue_id.to_string(),
            workflow_id: workflow_id.to_string(),
            execution_key: execution_key.to_string(),
            failure_reason: failure_reason.to_string(),
            failed_at: Utc::now().to_rfc3339(),
            attempt_count,
        };

        self.repo.insert(&item)?;
        Ok(item)
    }

    pub fn list_dlq(&self, limit: usize) -> Result<Vec<DeadLetterItem>, AresError> {
        self.repo.list(limit)
    }
}
