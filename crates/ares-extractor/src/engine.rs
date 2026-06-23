//! ExtractionEngine — the core orchestrator for knowledge extraction.
//!
//! Pipeline:
//!   1. Read commit from git
//!   2. Send to ExtractorProvider (LLM or mock)
//!   3. Filter candidates by confidence threshold
//!   4. Persist qualifying candidates to SQLite (knowledge_candidates table)
//!   5. Persist qualifying candidates as ARES memories
//!   6. Return ExtractionResult

use crate::git;
use crate::provider::ExtractorProvider;
use ares_core::{
    AresError, CreateMemoryInput, ExtractionConfig, ExtractionResult, ImportanceLevel,
    KnowledgeCandidate, KnowledgeType, MemorySource, MemoryType, ProjectId,
};
use ares_store::Store;
use std::path::Path;
use tracing::{debug, info, warn};

/// The main extraction engine that orchestrates git → LLM → store pipeline.
pub struct ExtractionEngine {
    store: Store,
    provider: Box<dyn ExtractorProvider>,
    config: ExtractionConfig,
}

impl ExtractionEngine {
    pub fn new(
        store: Store,
        provider: Box<dyn ExtractorProvider>,
        config: ExtractionConfig,
    ) -> Self {
        Self {
            store,
            provider,
            config,
        }
    }

    /// Extract knowledge from a specific commit in a repository.
    ///
    /// If `commit_hash` is None, defaults to HEAD.
    /// If `project_id` is provided, extracted knowledge is associated with that project.
    pub async fn extract_from_commit(
        &self,
        repo_path: &Path,
        commit_hash: Option<&str>,
        project_id: Option<&str>,
    ) -> Result<ExtractionResult, AresError> {
        // Step 1: Read commit info from git
        let commit_info = git::get_commit_info(repo_path, commit_hash)?;
        info!(
            commit = %commit_info.hash,
            files = commit_info.files_changed.len(),
            "Extracting knowledge from commit"
        );

        // Step 2: Send to the LLM provider
        let mut all_candidates = self
            .provider
            .extract(&commit_info)
            .await
            .map_err(|e| AresError::db(format!("Extraction provider failed: {e}")))?;

        // Limit candidates per commit
        all_candidates.truncate(self.config.max_candidates_per_commit);

        // Step 3: Filter by confidence threshold
        let threshold = self.config.confidence_threshold;
        let mut persisted_candidates = Vec::new();
        let mut rejected_count = 0usize;

        for candidate in &mut all_candidates {
            if candidate.confidence >= threshold {
                // Step 4: Persist to knowledge_candidates table
                self.persist_candidate(candidate)?;

                // Step 5: Persist as ARES memory if project_id is available
                if let Some(pid) = project_id {
                    self.persist_as_memory(candidate, pid)?;
                }

                candidate.persisted = true;
                persisted_candidates.push(candidate.clone());
            } else {
                debug!(
                    candidate_id = %candidate.id,
                    confidence = candidate.confidence,
                    threshold = threshold,
                    "Candidate rejected: below confidence threshold"
                );
                rejected_count += 1;
            }
        }

        info!(
            commit = %commit_info.hash,
            total = all_candidates.len(),
            persisted = persisted_candidates.len(),
            rejected = rejected_count,
            "Knowledge extraction complete"
        );

        Ok(ExtractionResult {
            commit_hash: commit_info.hash,
            commit_message: commit_info.message,
            all_candidates,
            persisted_candidates,
            confidence_threshold: threshold,
            rejected_count,
        })
    }

    /// Persist a KnowledgeCandidate to the knowledge_candidates table.
    fn persist_candidate(&self, candidate: &KnowledgeCandidate) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let affected_files_json =
            serde_json::to_string(&candidate.affected_files).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "INSERT INTO knowledge_candidates
                (id, knowledge_type, confidence, reasoning, content, title,
                 source_commit, affected_files, persisted, extracted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                candidate.id,
                candidate.knowledge_type.as_str(),
                candidate.confidence,
                candidate.reasoning,
                candidate.content,
                candidate.title,
                candidate.source_commit,
                affected_files_json,
                1, // persisted = true
                candidate.extracted_at,
            ],
        )
        .map_err(AresError::db)?;

        debug!(candidate_id = %candidate.id, "Knowledge candidate persisted");
        Ok(())
    }

    /// Persist a KnowledgeCandidate as an ARES Memory record.
    fn persist_as_memory(
        &self,
        candidate: &KnowledgeCandidate,
        project_id: &str,
    ) -> Result<(), AresError> {
        let memory_type = match candidate.knowledge_type {
            KnowledgeType::Decision => MemoryType::Decision,
            KnowledgeType::Bug => MemoryType::Bug,
            KnowledgeType::Architecture => MemoryType::Architecture,
            KnowledgeType::Experiment => MemoryType::Experiment,
        };

        let importance = match candidate.knowledge_type {
            KnowledgeType::Decision => ImportanceLevel::High,
            KnowledgeType::Bug => ImportanceLevel::High,
            KnowledgeType::Architecture => ImportanceLevel::Critical,
            KnowledgeType::Experiment => ImportanceLevel::Medium,
        };

        let content = serde_json::json!({
            "text": candidate.content,
            "reasoning": candidate.reasoning,
            "source_commit": candidate.source_commit,
            "affected_files": candidate.affected_files,
            "extraction_confidence": candidate.confidence,
        });

        let input = CreateMemoryInput {
            project_id: ProjectId::from(project_id.to_string()),
            memory_type,
            title: candidate.title.clone(),
            content,
            confidence: Some(candidate.confidence),
            importance: Some(importance),
            source: Some(MemorySource::Agent),
            ai_assisted: Some(true),
        };

        let repo = ares_store::SqliteMemoryRepository::new(self.store.clone());
        match repo.create(input) {
            Ok(memory) => {
                info!(
                    memory_id = %memory.id,
                    knowledge_type = %candidate.knowledge_type,
                    title = %candidate.title,
                    "Knowledge persisted as ARES memory"
                );
                Ok(())
            }
            Err(e) => {
                warn!(error = %e, "Failed to persist knowledge as memory, continuing");
                Ok(()) // Non-fatal — the candidate is already in knowledge_candidates
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::MockExtractorProvider;
    use ares_store::db::test_helpers::test_store;

    fn setup_project(store: &Store) -> String {
        let conn = store.get_conn().unwrap();
        let id = uuid::Uuid::now_v7().to_string();
        conn.execute(
            "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
             VALUES (?1, 'Test Project', 'Test', '/tmp/test', 'rust', 'test', 'greenfield', 0, 0)",
            rusqlite::params![id],
        ).unwrap();
        id
    }

    #[tokio::test]
    async fn test_extraction_engine_with_mock_provider() {
        let (store, _dir) = test_store();
        let project_id = setup_project(&store);

        let repo_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();

        if !repo_path.join(".git").exists() {
            return; // Skip in non-git environments
        }

        let config = ExtractionConfig::default();
        let provider = Box::new(MockExtractorProvider);
        let engine = ExtractionEngine::new(store.clone(), provider, config);

        let result = engine
            .extract_from_commit(repo_path, None, Some(&project_id))
            .await
            .unwrap();

        assert!(!result.commit_hash.is_empty());
        assert!(!result.all_candidates.is_empty());

        // Verify persisted candidates are in the database
        let conn = store.get_conn().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM knowledge_candidates", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(count as usize, result.persisted_candidates.len());
    }

    #[tokio::test]
    async fn test_confidence_threshold_filtering() {
        let (store, _dir) = test_store();

        let repo_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();

        if !repo_path.join(".git").exists() {
            return;
        }

        // Set threshold very high so most candidates are rejected
        let config = ExtractionConfig {
            confidence_threshold: 0.99,
            max_candidates_per_commit: 10,
        };
        let provider = Box::new(MockExtractorProvider);
        let engine = ExtractionEngine::new(store, provider, config);

        let result = engine
            .extract_from_commit(repo_path, None, None)
            .await
            .unwrap();

        // With 0.99 threshold, mock candidates (max 0.92) should all be rejected
        assert!(result.persisted_candidates.is_empty());
        assert!(result.rejected_count > 0);
    }
}
