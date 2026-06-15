use ares_core::{AresError, Memory, MemoryFilter, ProjectId};
use ares_store::repositories::memory::SqliteMemoryRepository;
use std::sync::Arc;

pub struct MemoryRetriever {
    repo: Arc<SqliteMemoryRepository>,
}

impl MemoryRetriever {
    pub fn new(repo: Arc<SqliteMemoryRepository>) -> Self {
        Self { repo }
    }

    /// Fetches all active memories for the project
    pub async fn fetch_project_memories(&self, project_id: &ProjectId) -> Result<Vec<Memory>, AresError> {
        // Just fetch all or apply a filter. Assuming a filter for now.
        let filter = MemoryFilter {
            memory_type: None,
            status: Some(ares_core::types::memory::MemoryStatus::Active),
            source: None,
            since: None,
            until: None,
        };
        let page = ares_core::types::pagination::Pagination { page: 1, page_size: 100 };
        let result = self.repo.list(project_id, filter, page)?;
        Ok(result.items)
    }
}
