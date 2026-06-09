use super::{models::*, repository::ExecutionRepository};
use ares_core::AresError;
use std::sync::Arc;

pub struct ExecutionService {
    repo: Arc<ExecutionRepository>,
}

impl ExecutionService {
    pub fn new(repo: Arc<ExecutionRepository>) -> Self {
        Self { repo }
    }

    pub fn list_executions(&self, limit: usize) -> Result<Vec<DistributedExecution>, AresError> {
        self.repo.list_executions(limit)
    }
}
