use ares_core::{AresError, MemoryType, ProjectId};
use ares_store::repositories::memory::SqliteMemoryRepository;
use std::sync::Arc;

pub struct SummaryRetriever {
    repo: Arc<SqliteMemoryRepository>,
}

impl SummaryRetriever {
    pub fn new(repo: Arc<SqliteMemoryRepository>) -> Self {
        Self { repo }
    }

    /// Retrieves the latest repository summary
    pub async fn fetch_latest_summary(
        &self,
        project_id: &ProjectId,
    ) -> Result<Option<String>, AresError> {
        let filter = ares_core::MemoryFilter {
            memory_type: Some(MemoryType::RepositorySummary),
            status: Some(ares_core::types::memory::MemoryStatus::Active),
            source: None,
            since: None,
            until: None,
        };
        let page = ares_core::types::pagination::Pagination {
            page: 1,
            page_size: 1,
        };
        let result = self.repo.list(project_id, filter, page)?;
        if let Some(mem) = result.items.first() {
            if let Some(content) = mem.content.as_str() {
                return Ok(Some(content.to_string()));
            }
        }
        Ok(None)
    }
}
