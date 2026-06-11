use super::models::*;
use super::repository::EpisodeRepository;
use ares_core::AresError;
use ares_store::db::Store;
use chrono::Utc;
use tracing::debug;
use uuid::Uuid;

/// Engine for episodic memory operations — store, query, rank, summarize.
pub struct EpisodicMemoryEngine {
    repo: EpisodeRepository,
}

impl EpisodicMemoryEngine {
    pub fn new(store: Store) -> Self {
        Self {
            repo: EpisodeRepository::new(store),
        }
    }

    /// Store a complete episode.
    pub fn store_episode(&self, episode: &Episode) -> Result<(), AresError> {
        debug!(episode_id = %episode.id, "Storing episode");
        self.repo.insert_episode(episode)
    }

    /// Store an event within an episode.
    pub fn store_event(&self, event: &EpisodeEvent) -> Result<(), AresError> {
        self.repo.insert_event(event)
    }

    /// Query episodes with filters.
    pub fn query_episodes(&self, query: &EpisodeQuery) -> Result<Vec<Episode>, AresError> {
        self.repo.query_episodes(query)
    }

    /// Find episodes similar to the given description and tags.
    pub fn find_similar(
        &self,
        description: &str,
        tags: &[String],
        limit: u32,
    ) -> Result<Vec<Episode>, AresError> {
        self.repo.find_similar(description, tags, limit)
    }

    /// Rank episodes by a composite score combining recency, relevance, and outcome quality.
    pub fn rank_episodes(&self, episodes: &[Episode]) -> Vec<RankedEpisode> {
        let now = Utc::now();
        let mut ranked: Vec<RankedEpisode> = episodes
            .iter()
            .map(|ep| {
                // Recency score: exponential decay over days
                let age_days = (now - ep.created_at).num_hours() as f64 / 24.0;
                let recency = (-0.01 * age_days).exp(); // 0.99^days

                // Outcome quality
                let outcome_score = match ep.outcome {
                    EpisodeOutcome::Success => 1.0,
                    EpisodeOutcome::PartialSuccess => 0.7,
                    EpisodeOutcome::Failure => 0.3, // failures are still valuable for learning
                    EpisodeOutcome::Aborted => 0.1,
                    EpisodeOutcome::Unknown => 0.2,
                };

                // Lessons learned boost
                let lessons_boost = (ep.lessons_learned.len() as f64 * 0.05).min(0.3);

                let relevance =
                    0.4 * recency + 0.3 * outcome_score + 0.2 * ep.score + 0.1 * lessons_boost;

                RankedEpisode {
                    episode: ep.clone(),
                    relevance_score: relevance,
                }
            })
            .collect();

        ranked.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        ranked
    }

    /// Summarize an episode from its events.
    pub fn summarize_episode(&self, episode_id: &str) -> Result<EpisodeSummary, AresError> {
        let episode = self
            .repo
            .get_episode(episode_id)?
            .ok_or_else(|| AresError::not_found("episode", episode_id))?;

        let events = self.repo.get_events(episode_id)?;

        // Build summary from episode data and events
        let event_descriptions: Vec<&str> = events.iter().map(|e| e.description.as_str()).collect();
        let total_chars: usize = event_descriptions.iter().map(|d| d.len()).sum();

        // Extract key events (milestones, decisions, errors)
        let key_events: Vec<String> = events
            .iter()
            .filter(|e| {
                matches!(
                    e.event_type,
                    EpisodeEventType::Milestone
                        | EpisodeEventType::Decision
                        | EpisodeEventType::Error
                )
            })
            .map(|e| e.description.clone())
            .collect();

        let summary_text = format!(
            "Mission '{}': {} with {} events. {}",
            episode.title,
            episode.outcome.as_str(),
            events.len(),
            if episode.lessons_learned.is_empty() {
                String::new()
            } else {
                format!("Key lessons: {}", episode.lessons_learned.join("; "))
            }
        );

        let summary_chars = summary_text.len();
        let compression_ratio = if total_chars > 0 {
            summary_chars as f64 / total_chars as f64
        } else {
            1.0
        };

        let mut key_insights = key_events;
        key_insights.extend(episode.lessons_learned.iter().cloned());
        key_insights.dedup();

        let summary = EpisodeSummary {
            id: Uuid::now_v7().to_string(),
            episode_id: episode_id.into(),
            summary_text,
            key_insights,
            compression_ratio: compression_ratio.min(1.0),
            created_at: Utc::now(),
        };

        self.repo.upsert_summary(&summary)?;
        Ok(summary)
    }

    /// Get an existing summary.
    pub fn get_summary(&self, episode_id: &str) -> Result<Option<EpisodeSummary>, AresError> {
        self.repo.get_summary(episode_id)
    }

    /// Get an episode by ID.
    pub fn get_episode(&self, id: &str) -> Result<Option<Episode>, AresError> {
        self.repo.get_episode(id)
    }

    /// Count total episodes.
    pub fn count(&self) -> Result<u64, AresError> {
        self.repo.count_episodes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episodic::repository::make_test_episode;
    use crate::test_utils::test_store;

    fn make_engine() -> (EpisodicMemoryEngine, tempfile::TempDir) {
        let (store, dir) = test_store();
        (EpisodicMemoryEngine::new(store), dir)
    }

    #[test]
    fn store_and_retrieve_episode() {
        let (engine, _dir) = make_engine();
        let ep = make_test_episode("ep_eng_1", EpisodeOutcome::Success);
        engine.store_episode(&ep).unwrap();

        let fetched = engine.get_episode("ep_eng_1").unwrap().unwrap();
        assert_eq!(fetched.title, ep.title);
    }

    #[test]
    fn rank_episodes_orders_by_relevance() {
        let (engine, _dir) = make_engine();

        let ep_success = make_test_episode("ep_s", EpisodeOutcome::Success);
        let ep_failure = make_test_episode("ep_f", EpisodeOutcome::Failure);

        let ranked = engine.rank_episodes(&[ep_success, ep_failure]);
        assert_eq!(ranked.len(), 2);
        // Success should rank higher than failure (assuming same recency/score)
        assert!(ranked[0].relevance_score >= ranked[1].relevance_score);
    }

    #[test]
    fn rank_empty_list() {
        let (engine, _dir) = make_engine();
        let ranked = engine.rank_episodes(&[]);
        assert!(ranked.is_empty());
    }

    #[test]
    fn summarize_episode_with_events() {
        let (engine, _dir) = make_engine();
        let ep = make_test_episode("ep_sum_eng", EpisodeOutcome::Success);
        engine.store_episode(&ep).unwrap();

        // Add some events
        let events = vec![
            EpisodeEvent {
                id: Uuid::now_v7().to_string(),
                episode_id: "ep_sum_eng".into(),
                event_type: EpisodeEventType::Action,
                description: "Started implementation".into(),
                agent_id: Some("agent_1".into()),
                timestamp: Utc::now(),
                metadata: serde_json::json!({}),
            },
            EpisodeEvent {
                id: Uuid::now_v7().to_string(),
                episode_id: "ep_sum_eng".into(),
                event_type: EpisodeEventType::Milestone,
                description: "Core logic complete".into(),
                agent_id: None,
                timestamp: Utc::now(),
                metadata: serde_json::json!({}),
            },
            EpisodeEvent {
                id: Uuid::now_v7().to_string(),
                episode_id: "ep_sum_eng".into(),
                event_type: EpisodeEventType::Decision,
                description: "Chose async approach".into(),
                agent_id: Some("agent_1".into()),
                timestamp: Utc::now(),
                metadata: serde_json::json!({}),
            },
        ];

        for ev in &events {
            engine.store_event(ev).unwrap();
        }

        let summary = engine.summarize_episode("ep_sum_eng").unwrap();
        assert!(!summary.summary_text.is_empty());
        assert!(summary.summary_text.contains("success"));
        // Key insights should include milestone and decision descriptions
        assert!(!summary.key_insights.is_empty());
    }

    #[test]
    fn summarize_nonexistent_episode_fails() {
        let (engine, _dir) = make_engine();
        let result = engine.summarize_episode("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn query_through_engine() {
        let (engine, _dir) = make_engine();
        engine
            .store_episode(&make_test_episode("ep_q1", EpisodeOutcome::Success))
            .unwrap();
        engine
            .store_episode(&make_test_episode("ep_q2", EpisodeOutcome::Failure))
            .unwrap();

        let query = EpisodeQuery {
            outcome: Some(EpisodeOutcome::Failure),
            ..Default::default()
        };
        let results = engine.query_episodes(&query).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn find_similar_through_engine() {
        let (engine, _dir) = make_engine();
        let mut ep = make_test_episode("ep_sim", EpisodeOutcome::Success);
        ep.description = "Deploying microservice to kubernetes".into();
        ep.tags = vec!["deploy".into(), "k8s".into()];
        engine.store_episode(&ep).unwrap();

        let results = engine
            .find_similar("Deploying application", &["deploy".into()], 10)
            .unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn count_through_engine() {
        let (engine, _dir) = make_engine();
        assert_eq!(engine.count().unwrap(), 0);
        engine
            .store_episode(&make_test_episode("ep_cnt", EpisodeOutcome::Success))
            .unwrap();
        assert_eq!(engine.count().unwrap(), 1);
    }

    #[test]
    fn get_summary_returns_none_when_missing() {
        let (engine, _dir) = make_engine();
        let ep = make_test_episode("ep_no_sum", EpisodeOutcome::Success);
        engine.store_episode(&ep).unwrap();
        let result = engine.get_summary("ep_no_sum").unwrap();
        assert!(result.is_none());
    }
}
