use crate::legacy::modes::RetrievalMode;
use crate::legacy::ranking::RetrievalRanker;
use crate::legacy::types::RankedResult;
use ares_agent::services::semantic_retrieval::SemanticSearchService;
use ares_core::id::ProjectId;
use ares_core::types::memory::{Memory, MemoryFilter};
use ares_core::Pagination;
use ares_store::repositories::memory::SqliteMemoryRepository;
use std::sync::Arc;

pub struct RetrievalEngine {
    memory_repo: Arc<SqliteMemoryRepository>,
    semantic_search: Arc<SemanticSearchService>,
}

impl RetrievalEngine {
    pub fn new(
        memory_repo: Arc<SqliteMemoryRepository>,
        semantic_search: Arc<SemanticSearchService>,
    ) -> Self {
        Self {
            memory_repo,
            semantic_search,
        }
    }

    pub async fn retrieve(
        &self,
        project_id: &ProjectId,
        query: &str,
        mode: RetrievalMode,
        limit: usize,
    ) -> Result<Vec<RankedResult>, anyhow::Error> {
        let candidates = if query.trim().is_empty() {
            // No query, just fetch recent or important based on mode
            let page = self
                .memory_repo
                .list(
                    project_id,
                    MemoryFilter::default(),
                    Pagination {
                        page: 1,
                        page_size: 1000,
                    },
                )
                .map_err(|e| anyhow::anyhow!("DB Error: {}", e))?;
            let filtered: Vec<(Memory, f32)> = page
                .items
                .into_iter()
                .filter(|m| self.matches_mode(m, &mode))
                .map(|m| (m, 0.5)) // Default semantic score
                .collect();
            filtered
        } else {
            // Query present, use semantic search
            let search_response = self
                .semantic_search
                .search(project_id, query, limit)
                .await
                .map_err(|e| anyhow::anyhow!("Search error: {}", e))?;
            search_response
                .results
                .into_iter()
                .filter(|r| self.matches_mode(&r.memory, &mode))
                .map(|r| (r.memory, r.final_score))
                .collect()
        };

        let ranker = RetrievalRanker::new(mode);
        let mut ranked = ranker.rank(&candidates);

        ranked.truncate(limit);
        Ok(ranked)
    }

    fn matches_mode(&self, memory: &Memory, mode: &RetrievalMode) -> bool {
        let stype = memory.memory_type.to_string().to_lowercase();
        match mode {
            RetrievalMode::General
            | RetrievalMode::RecentContext
            | RetrievalMode::ProjectSummary => true,
            RetrievalMode::DecisionHistory => stype.contains("decision"),
            RetrievalMode::BugHistory => stype.contains("bug") || stype.contains("error"),
            RetrievalMode::FeatureHistory => stype.contains("feature"),
            RetrievalMode::ArchitectureContext => {
                stype.contains("architecture") || stype.contains("design")
            }
        }
    }
}
