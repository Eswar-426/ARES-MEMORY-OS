use crate::services::memory_ranking::MemoryRankingEngine;
use ares_core::{AresError, Memory, ProjectId};
use ares_store::repositories::intelligence::SqliteIntelligenceRepository;
use ares_store::repositories::memory::SqliteMemoryRepository;
use std::sync::Arc;

pub struct SemanticRetrievalLayer {
    memory_repo: Arc<SqliteMemoryRepository>,
    intelligence_repo: Arc<SqliteIntelligenceRepository>,
    ranking_engine: Arc<MemoryRankingEngine>,
}

impl SemanticRetrievalLayer {
    pub fn new(
        memory_repo: Arc<SqliteMemoryRepository>,
        intelligence_repo: Arc<SqliteIntelligenceRepository>,
        ranking_engine: Arc<MemoryRankingEngine>,
    ) -> Self {
        Self {
            memory_repo,
            intelligence_repo,
            ranking_engine,
        }
    }

    pub fn retrieve(
        &self,
        project_id: &ProjectId,
        query: &str,
        limit: u32,
    ) -> Result<Vec<Memory>, AresError> {
        // 1. FTS Search
        let fts_results = self.memory_repo.search(project_id, query, limit * 2)?;

        let mut candidate_memories = Vec::new();
        let mut relevance_scores = Vec::new();
        let mut access_counts = Vec::new();

        for result in fts_results {
            let mem_id = result.memory.id.clone();

            // Log access for retrieval context
            self.intelligence_repo.log_access(
                project_id,
                &mem_id,
                ares_core::AccessContext::Retrieval,
            )?;

            // Get historical access count
            let count = self.intelligence_repo.get_access_count(&mem_id)?;
            access_counts.push((mem_id.clone(), count));

            relevance_scores.push((mem_id, result.score as f32));
            candidate_memories.push(result.memory);
        }

        // 2. Ranking Pipeline
        let ranked = self.ranking_engine.rank_memories(
            &candidate_memories,
            &relevance_scores,
            &access_counts,
        );

        // 3. Cache ranks and return top results
        let mut final_results = Vec::new();
        for (memory, score) in ranked.into_iter().take(limit as usize) {
            self.intelligence_repo
                .set_ranking(project_id, &memory.id, score.final_score)?;
            final_results.push(memory);
        }

        Ok(final_results)
    }
}
