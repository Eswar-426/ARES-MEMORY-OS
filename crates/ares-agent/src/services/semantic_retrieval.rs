//! Semantic retrieval service — the full search pipeline.
//!
//! Orchestrates: query embedding → vector search → keyword search →
//! graph expansion → hybrid ranking → diagnostics.
//!
//! Designed for async-ready embedding generation: memory creation does NOT
//! block on embedding generation.  Embeddings can be generated lazily or
//! by a background worker.

use crate::services::hybrid_ranking::{
    HybridRankingConfig, HybridRankingEngine, SemanticSearchResult,
};
use ares_core::vector::{
    traits::{EmbeddingProvider, VectorRepository},
    types::RetrievalDiagnostics,
};
use ares_core::{AresError, Memory, MemoryId, ProjectId};
use ares_store::repositories::memory::SqliteMemoryRepository;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

// ─────────────────────────────────────────────────────────────────
// Semantic Search Service
// ─────────────────────────────────────────────────────────────────

/// The top-level semantic search service.
///
/// Combines vector similarity, keyword search, graph expansion, and
/// hybrid ranking into a single pipeline.
pub struct SemanticSearchService {
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_repo: Arc<dyn VectorRepository>,
    memory_repo: Arc<SqliteMemoryRepository>,
    ranking_engine: HybridRankingEngine,
}

/// Combined result of a semantic search including diagnostics.
#[derive(Debug, Clone)]
pub struct SemanticSearchResponse {
    pub results: Vec<SemanticSearchResult>,
    pub diagnostics: RetrievalDiagnostics,
}

impl SemanticSearchService {
    pub fn new(
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_repo: Arc<dyn VectorRepository>,
        memory_repo: Arc<SqliteMemoryRepository>,
        ranking_config: HybridRankingConfig,
    ) -> Self {
        Self {
            embedding_provider,
            vector_repo,
            memory_repo,
            ranking_engine: HybridRankingEngine::new(ranking_config),
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Semantic Search Pipeline
    // ─────────────────────────────────────────────────────────────

    /// Execute a full semantic search.
    ///
    /// Pipeline:
    /// 1. Generate query embedding
    /// 2. Vector search (top limit×3 candidates)
    /// 3. Keyword search (FTS5, top limit×2 candidates)
    /// 4. Merge + deduplicate
    /// 5. Hybrid re-ranking
    /// 6. Return top N with diagnostics
    pub async fn search(
        &self,
        project_id: &ProjectId,
        query: &str,
        limit: usize,
    ) -> Result<SemanticSearchResponse, AresError> {
        let pipeline_start = Instant::now();
        let metadata = self.embedding_provider.metadata();

        let mut diagnostics = RetrievalDiagnostics {
            provider: metadata.provider.clone(),
            model: metadata.model.clone(),
            ..Default::default()
        };

        // 1. Generate query embedding
        let embed_start = Instant::now();
        let query_embedding = self.embedding_provider.embed(query).await?;
        diagnostics.embedding_latency_ms = embed_start.elapsed().as_secs_f64() * 1000.0;

        // 2. Vector search
        let vec_start = Instant::now();
        let vector_results =
            self.vector_repo
                .search_similar(&query_embedding.vector, &metadata, limit * 3)?;
        diagnostics.vector_search_latency_ms = vec_start.elapsed().as_secs_f64() * 1000.0;
        diagnostics.vector_candidates = vector_results.len();

        debug!(
            vector_candidates = vector_results.len(),
            "Vector search complete"
        );

        // 3. Keyword search (FTS5)
        let kw_start = Instant::now();
        let keyword_results = self
            .memory_repo
            .search(project_id, query, (limit * 2) as u32)?;
        diagnostics.keyword_search_latency_ms = kw_start.elapsed().as_secs_f64() * 1000.0;
        diagnostics.keyword_candidates = keyword_results.len();

        // 4. Merge + deduplicate
        let mut candidate_map: HashMap<MemoryId, Memory> = HashMap::new();
        let mut semantic_scores: HashMap<MemoryId, f32> = HashMap::new();
        let mut keyword_scores: HashMap<MemoryId, f32> = HashMap::new();

        // Add vector results
        for vr in &vector_results {
            let mem_id = MemoryId::from(vr.memory_id.clone());
            semantic_scores.insert(mem_id.clone(), vr.score);

            // Load memory from DB if not yet loaded
            if let std::collections::hash_map::Entry::Vacant(e) =
                candidate_map.entry(mem_id.clone())
            {
                if let Ok(Some(memory)) = self.memory_repo.get_by_id(&mem_id) {
                    // Only include active memories for the right project
                    if memory.project_id == *project_id && memory.deleted_at.is_none() {
                        e.insert(memory);
                    }
                }
            }
        }

        // Add keyword results
        // Normalize FTS scores to [0, 1] range
        let max_kw_score = keyword_results
            .iter()
            .map(|r| r.score)
            .fold(0.0_f64, f64::max);
        let norm_factor = if max_kw_score > 0.0 {
            1.0 / max_kw_score
        } else {
            0.0
        };

        for kr in &keyword_results {
            let mem_id = kr.memory.id.clone();
            keyword_scores.insert(mem_id.clone(), (kr.score * norm_factor) as f32);
            candidate_map
                .entry(mem_id)
                .or_insert_with(|| kr.memory.clone());
        }

        let candidates: Vec<Memory> = candidate_map.into_values().collect();
        diagnostics.merged_candidates = candidates.len();

        debug!(
            merged_candidates = candidates.len(),
            "Candidates merged and deduplicated"
        );

        // 5. Hybrid re-ranking (graph scores are empty for now — future expansion)
        let rank_start = Instant::now();
        let graph_scores: HashMap<MemoryId, f32> = HashMap::new();
        let ranked = self.ranking_engine.rank(
            &candidates,
            &semantic_scores,
            &keyword_scores,
            &graph_scores,
        );
        diagnostics.ranking_latency_ms = rank_start.elapsed().as_secs_f64() * 1000.0;

        // 6. Take top N
        let results: Vec<SemanticSearchResult> = ranked.into_iter().take(limit).collect();
        diagnostics.results_returned = results.len();
        diagnostics.total_latency_ms = pipeline_start.elapsed().as_secs_f64() * 1000.0;

        info!(
            results = diagnostics.results_returned,
            total_ms = format!("{:.1}", diagnostics.total_latency_ms),
            "Semantic search complete"
        );

        Ok(SemanticSearchResponse {
            results,
            diagnostics,
        })
    }

    // ─────────────────────────────────────────────────────────────
    // Embedding Management (async-ready)
    // ─────────────────────────────────────────────────────────────

    /// Generate and store an embedding for a single memory.
    ///
    /// This is designed to be called asynchronously — memory creation
    /// does not need to wait for this to complete.
    pub async fn embed_memory(&self, memory: &Memory) -> Result<(), AresError> {
        let text = format_memory_for_embedding(memory);
        let embedding = self.embedding_provider.embed(&text).await?;
        let metadata = self.embedding_provider.metadata();

        self.vector_repo
            .upsert_embedding(memory.id.as_str(), &embedding.vector, &metadata)?;

        debug!(memory_id = %memory.id, dims = embedding.dimensions, "Memory embedded");
        Ok(())
    }

    /// Delete the embedding for a memory.
    pub fn delete_memory_embedding(&self, memory_id: &MemoryId) -> Result<(), AresError> {
        self.vector_repo.delete_embedding(memory_id.as_str())
    }

    /// Re-index all memories for a project: regenerate all embeddings.
    ///
    /// Returns the number of memories re-indexed.
    pub async fn reindex_project(&self, project_id: &ProjectId) -> Result<u32, AresError> {
        use ares_core::{MemoryFilter, Pagination};

        info!(project_id = %project_id, "Starting full re-index");

        let mut count = 0u32;
        let mut current_page = 1;

        loop {
            let page = self.memory_repo.list(
                project_id,
                MemoryFilter::default(),
                Pagination {
                    page: current_page,
                    page_size: 100,
                },
            )?;

            if page.items.is_empty() {
                break;
            }

            for memory in &page.items {
                match self.embed_memory(memory).await {
                    Ok(()) => count += 1,
                    Err(e) => {
                        warn!(memory_id = %memory.id, error = %e, "Failed to embed memory during reindex");
                    }
                }
            }

            if current_page >= page.total_pages {
                break;
            }
            current_page += 1;
        }

        info!(project_id = %project_id, reindexed = count, "Re-index complete");
        Ok(count)
    }

    /// Get the embedding provider metadata.
    pub fn provider_metadata(&self) -> ares_core::EmbeddingMetadata {
        self.embedding_provider.metadata()
    }

    /// Get the number of stored embeddings.
    pub fn embedding_count(&self) -> Result<u64, AresError> {
        self.vector_repo.count()
    }
}

// ─────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────

/// Format a memory into a text string suitable for embedding.
///
/// Combines title + content into a single string for the embedding model.
fn format_memory_for_embedding(memory: &Memory) -> String {
    let content_str = match &memory.content {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    };
    format!(
        "{} — {} [{}]",
        memory.title,
        content_str,
        memory.memory_type.as_str()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ares_core::*;

    #[test]
    fn format_memory_combines_title_and_content() {
        let memory = Memory {
            id: MemoryId::from("m1"),
            project_id: ProjectId::from("p1"),
            memory_type: MemoryType::Feature,
            title: "JWT Auth".into(),
            content: serde_json::json!({"desc": "token validation"}),
            status: MemoryStatus::Active,
            version: 1,
            parent_id: None,
            confidence: 1.0,
            importance: ImportanceLevel::High,
            source: MemorySource::Human,
            ai_assisted: false,
            created_at: 0,
            updated_at: 0,
            deleted_at: None,
        };

        let text = format_memory_for_embedding(&memory);
        assert!(text.contains("JWT Auth"));
        assert!(text.contains("token validation"));
        assert!(text.contains("feature"));
    }
}
