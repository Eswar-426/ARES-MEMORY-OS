use super::models::*;
use super::repository::RetrievalRepository;
use crate::episodic::models::{Episode, EpisodeOutcome, EpisodeQuery};
use crate::episodic::repository::EpisodeRepository;
use crate::experience::models::Principle;
use crate::experience::repository::ExperienceRepository;
use crate::semantic::models::{SemanticMemory, SemanticQuery};
use crate::semantic::repository::SemanticRepository;
use ares_core::AresError;
use ares_store::db::Store;
use chrono::Utc;
use std::time::Instant;
use tracing::debug;
use uuid::Uuid;

/// Retrieval-augmented memory engine — unified search across all memory types.
pub struct RetrievalEngine {
    episode_repo: EpisodeRepository,
    semantic_repo: SemanticRepository,
    experience_repo: ExperienceRepository,
    retrieval_repo: RetrievalRepository,
}

impl RetrievalEngine {
    pub fn new(store: Store) -> Self {
        Self {
            episode_repo: EpisodeRepository::new(store.clone()),
            semantic_repo: SemanticRepository::new(store.clone()),
            experience_repo: ExperienceRepository::new(store.clone()),
            retrieval_repo: RetrievalRepository::new(store),
        }
    }

    /// Search across all memory types based on the request.
    pub fn retrieve(&self, request: &RetrievalRequest) -> Result<RetrievalResponse, AresError> {
        let start = Instant::now();
        debug!(query_type = ?request.query_type, "Retrieval request");

        let mut all_results = Vec::new();

        match request.query_type {
            RetrievalQueryType::SimilarMission => {
                all_results.extend(self.search_episodes(request)?);
                all_results.extend(self.search_semantic(request)?);
            }
            RetrievalQueryType::FailureSearch => {
                all_results.extend(self.search_failure_episodes(request)?);
                all_results.extend(self.search_semantic(request)?);
            }
            RetrievalQueryType::SuccessSearch => {
                all_results.extend(self.search_success_episodes(request)?);
                all_results.extend(self.search_semantic(request)?);
            }
            RetrievalQueryType::LessonSearch => {
                all_results.extend(self.search_episodes(request)?);
                all_results.extend(self.search_principles(request)?);
            }
            RetrievalQueryType::PrincipleSearch => {
                all_results.extend(self.search_principles(request)?);
            }
            RetrievalQueryType::General => {
                all_results.extend(self.search_episodes(request)?);
                all_results.extend(self.search_semantic(request)?);
                all_results.extend(self.search_principles(request)?);
            }
        }

        // Filter by min confidence
        if request.min_confidence > 0.0 {
            all_results.retain(|r| r.confidence >= request.min_confidence);
        }

        // Sort by relevance
        all_results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        all_results.truncate(request.max_results as usize);

        let elapsed = start.elapsed();
        let total_count = all_results.len();

        // Log the retrieval
        let result_ids: Vec<String> = all_results.iter().map(|r| r.id.clone()).collect();
        let avg_relevance = if all_results.is_empty() {
            0.0
        } else {
            all_results.iter().map(|r| r.relevance_score).sum::<f64>() / all_results.len() as f64
        };

        let log_entry = RetrievalLogEntry {
            id: Uuid::now_v7().to_string(),
            query_text: request.query_text.clone(),
            query_type: request.query_type.as_str().into(),
            results_count: total_count as u32,
            result_ids,
            relevance_score: avg_relevance,
            retrieval_ms: elapsed.as_millis() as u64,
            created_at: Utc::now().timestamp_micros(),
        };
        let _ = self.retrieval_repo.log_retrieval(&log_entry);

        Ok(RetrievalResponse {
            results: all_results,
            total_count,
            query_type: request.query_type.as_str().into(),
            retrieval_ms: elapsed.as_millis() as u64,
        })
    }

    fn search_episodes(
        &self,
        request: &RetrievalRequest,
    ) -> Result<Vec<RetrievalResult>, AresError> {
        let query = EpisodeQuery {
            search_text: Some(request.query_text.clone()),
            tags: request.tags.clone(),
            limit: Some(request.max_results),
            ..Default::default()
        };
        let episodes = self.episode_repo.query_episodes(&query)?;
        Ok(episodes_to_results(&episodes))
    }

    fn search_failure_episodes(
        &self,
        request: &RetrievalRequest,
    ) -> Result<Vec<RetrievalResult>, AresError> {
        let query = EpisodeQuery {
            outcome: Some(EpisodeOutcome::Failure),
            search_text: if request.query_text.is_empty() {
                None
            } else {
                Some(request.query_text.clone())
            },
            limit: Some(request.max_results),
            ..Default::default()
        };
        let episodes = self.episode_repo.query_episodes(&query)?;
        Ok(episodes_to_results(&episodes))
    }

    fn search_success_episodes(
        &self,
        request: &RetrievalRequest,
    ) -> Result<Vec<RetrievalResult>, AresError> {
        let query = EpisodeQuery {
            outcome: Some(EpisodeOutcome::Success),
            search_text: if request.query_text.is_empty() {
                None
            } else {
                Some(request.query_text.clone())
            },
            limit: Some(request.max_results),
            ..Default::default()
        };
        let episodes = self.episode_repo.query_episodes(&query)?;
        Ok(episodes_to_results(&episodes))
    }

    fn search_semantic(
        &self,
        request: &RetrievalRequest,
    ) -> Result<Vec<RetrievalResult>, AresError> {
        let query = SemanticQuery {
            subject: Some(request.query_text.clone()),
            min_confidence: Some(request.min_confidence),
            limit: Some(request.max_results),
            ..Default::default()
        };
        let memories = self.semantic_repo.query(&query)?;
        Ok(memories_to_results(&memories))
    }

    fn search_principles(
        &self,
        request: &RetrievalRequest,
    ) -> Result<Vec<RetrievalResult>, AresError> {
        let principles = self.experience_repo.list_active_principles(None)?;

        // Filter by query text
        let filtered: Vec<&Principle> = if request.query_text.is_empty() {
            principles.iter().collect()
        } else {
            let lower = request.query_text.to_lowercase();
            principles
                .iter()
                .filter(|p| {
                    p.title.to_lowercase().contains(&lower)
                        || p.description.to_lowercase().contains(&lower)
                        || p.domain.to_lowercase().contains(&lower)
                })
                .collect()
        };

        Ok(filtered
            .iter()
            .map(|p| RetrievalResult {
                id: p.id.clone(),
                source_type: "principle".into(),
                title: p.title.clone(),
                content: p.description.clone(),
                relevance_score: p.confidence,
                confidence: p.confidence,
            })
            .collect())
    }

    /// Get recent retrieval logs.
    pub fn recent_logs(&self, limit: u32) -> Result<Vec<RetrievalLogEntry>, AresError> {
        self.retrieval_repo.recent_logs(limit)
    }

    /// Count retrieval logs.
    pub fn count_logs(&self) -> Result<u64, AresError> {
        self.retrieval_repo.count()
    }
}

fn episodes_to_results(episodes: &[Episode]) -> Vec<RetrievalResult> {
    episodes
        .iter()
        .map(|ep| RetrievalResult {
            id: ep.id.clone(),
            source_type: "episode".into(),
            title: ep.title.clone(),
            content: format!(
                "{} [{}] lessons: {}",
                ep.description,
                ep.outcome.as_str(),
                ep.lessons_learned.join("; ")
            ),
            relevance_score: ep.score,
            confidence: ep.score,
        })
        .collect()
}

fn memories_to_results(memories: &[SemanticMemory]) -> Vec<RetrievalResult> {
    memories
        .iter()
        .map(|m| RetrievalResult {
            id: m.id.clone(),
            source_type: "semantic_memory".into(),
            title: m.subject.clone(),
            content: format!("{} {} {}", m.subject, m.predicate, m.object),
            relevance_score: m.confidence,
            confidence: m.confidence,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episodic::repository::make_test_episode;
    use crate::test_utils::test_store;

    fn make_engine() -> (RetrievalEngine, Store, tempfile::TempDir) {
        let (store, dir) = test_store();
        let engine = RetrievalEngine::new(store.clone());
        (engine, store, dir)
    }

    #[test]
    fn retrieve_empty_db() {
        let (engine, _, _dir) = make_engine();
        let request = RetrievalRequest {
            query_text: "deploy".into(),
            ..Default::default()
        };
        let response = engine.retrieve(&request).unwrap();
        assert!(response.results.is_empty());
    }

    #[test]
    fn retrieve_finds_episodes() {
        let (engine, store, _dir) = make_engine();
        let ep_repo = EpisodeRepository::new(store);
        let mut ep = make_test_episode("ep_ret", EpisodeOutcome::Success);
        ep.title = "Deploy microservice".into();
        ep.description = "Deployed to production cluster".into();
        ep_repo.insert_episode(&ep).unwrap();

        let request = RetrievalRequest {
            query_text: "Deploy".into(),
            query_type: RetrievalQueryType::SimilarMission,
            ..Default::default()
        };
        let response = engine.retrieve(&request).unwrap();
        assert!(!response.results.is_empty());
        assert_eq!(response.results[0].source_type, "episode");
    }

    #[test]
    fn retrieve_failure_search() {
        let (engine, store, _dir) = make_engine();
        let ep_repo = EpisodeRepository::new(store);
        ep_repo
            .insert_episode(&make_test_episode("ep_fail", EpisodeOutcome::Failure))
            .unwrap();

        let request = RetrievalRequest {
            query_text: "".into(),
            query_type: RetrievalQueryType::FailureSearch,
            ..Default::default()
        };
        let response = engine.retrieve(&request).unwrap();
        assert!(!response.results.is_empty());
    }

    #[test]
    fn retrieve_success_search() {
        let (engine, store, _dir) = make_engine();
        let ep_repo = EpisodeRepository::new(store);
        ep_repo
            .insert_episode(&make_test_episode("ep_succ", EpisodeOutcome::Success))
            .unwrap();

        let request = RetrievalRequest {
            query_text: "".into(),
            query_type: RetrievalQueryType::SuccessSearch,
            ..Default::default()
        };
        let response = engine.retrieve(&request).unwrap();
        assert!(!response.results.is_empty());
    }

    #[test]
    fn retrieve_respects_max_results() {
        let (engine, store, _dir) = make_engine();
        let ep_repo = EpisodeRepository::new(store);
        for i in 0..10 {
            ep_repo
                .insert_episode(&make_test_episode(
                    &format!("ep_lim_{}", i),
                    EpisodeOutcome::Success,
                ))
                .unwrap();
        }

        let request = RetrievalRequest {
            query_text: "Test".into(),
            max_results: 3,
            ..Default::default()
        };
        let response = engine.retrieve(&request).unwrap();
        assert!(response.results.len() <= 3);
    }

    #[test]
    fn retrieve_logs_query() {
        let (engine, store, _dir) = make_engine();
        let ep_repo = EpisodeRepository::new(store);
        ep_repo
            .insert_episode(&make_test_episode("ep_log", EpisodeOutcome::Success))
            .unwrap();

        let request = RetrievalRequest {
            query_text: "Test".into(),
            ..Default::default()
        };
        engine.retrieve(&request).unwrap();

        let logs = engine.recent_logs(10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].query_text, "Test");
    }

    #[test]
    fn retrieve_filters_by_min_confidence() {
        let (engine, store, _dir) = make_engine();
        let ep_repo = EpisodeRepository::new(store);
        let mut ep = make_test_episode("ep_conf", EpisodeOutcome::Success);
        ep.score = 0.2; // Low score
        ep_repo.insert_episode(&ep).unwrap();

        let request = RetrievalRequest {
            query_text: "Test".into(),
            min_confidence: 0.5,
            ..Default::default()
        };
        let response = engine.retrieve(&request).unwrap();
        // Should be filtered out due to low score
        assert!(response.results.is_empty());
    }

    #[test]
    fn retrieve_general_searches_all() {
        let (engine, _, _dir) = make_engine();
        let request = RetrievalRequest {
            query_text: "anything".into(),
            query_type: RetrievalQueryType::General,
            ..Default::default()
        };
        let response = engine.retrieve(&request).unwrap();
        // Should not error even with empty DB
        assert_eq!(response.query_type, "general");
    }

    #[test]
    fn count_logs() {
        let (engine, _, _dir) = make_engine();
        assert_eq!(engine.count_logs().unwrap(), 0);
    }

    #[test]
    fn episode_to_result_format() {
        let ep = make_test_episode("ep_fmt", EpisodeOutcome::Success);
        let results = episodes_to_results(&[ep]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source_type, "episode");
        assert!(results[0].content.contains("success"));
    }
}
