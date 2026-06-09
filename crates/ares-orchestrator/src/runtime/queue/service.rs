use super::{dto::EnqueueRequest, models::{QueueStatus, WorkflowQueueItem}, repository::QueueRepository};
use ares_core::AresError;
use chrono::Utc;
use uuid::Uuid;

pub struct QueueService {
    repo: QueueRepository,
}

impl QueueService {
    pub fn new(repo: QueueRepository) -> Self {
        Self { repo }
    }

    pub fn enqueue(&self, req: EnqueueRequest) -> Result<WorkflowQueueItem, AresError> {
        let now = Utc::now().to_rfc3339();
        
        let item = WorkflowQueueItem {
            id: Uuid::now_v7().to_string(),
            workflow_id: req.workflow_id,
            priority: req.priority,
            status: QueueStatus::Queued,
            assigned_worker: None,
            retry_count: 0,
            created_at: now,
            started_at: None,
            completed_at: None,
            execution_key: req.execution_key,
            execution_checksum: req.execution_checksum,
        };

        self.repo.enqueue(&item)?;
        Ok(item)
    }

    pub fn assign_worker(&self, queue_item_id: &str, worker_id: &str) -> Result<(), AresError> {
        self.repo.update_status(queue_item_id, &QueueStatus::Assigned, Some(worker_id), None, None)
    }
}
