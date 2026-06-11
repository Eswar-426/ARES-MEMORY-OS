use super::models::*;
use ares_core::AresError;
use ares_store::db::Store;
use chrono::{TimeZone, Utc};
use rusqlite::params;

/// SQLite-backed repository for episodic memory persistence.
pub struct EpisodeRepository {
    store: Store,
}

impl EpisodeRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    /// Insert a new episode into the database.
    pub fn insert_episode(&self, episode: &Episode) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let agents_json = serde_json::to_string(&episode.agents_involved)
            .map_err(|e| AresError::Serialization(e.to_string()))?;
        let decisions_json = serde_json::to_string(&episode.decisions_made)
            .map_err(|e| AresError::Serialization(e.to_string()))?;
        let failures_json = serde_json::to_string(&episode.failures)
            .map_err(|e| AresError::Serialization(e.to_string()))?;
        let lessons_json = serde_json::to_string(&episode.lessons_learned)
            .map_err(|e| AresError::Serialization(e.to_string()))?;
        let tags_json = serde_json::to_string(&episode.tags)
            .map_err(|e| AresError::Serialization(e.to_string()))?;
        let completed_at_ts = episode.completed_at.map(|dt| dt.timestamp_micros());

        conn.execute(
            "INSERT INTO episodes (id, mission_id, title, description, agents_involved,
             decisions_made, outcome, score, cost, duration_secs, failures,
             lessons_learned, tags, created_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                episode.id,
                episode.mission_id,
                episode.title,
                episode.description,
                agents_json,
                decisions_json,
                episode.outcome.as_str(),
                episode.score,
                episode.cost,
                episode.duration_secs,
                failures_json,
                lessons_json,
                tags_json,
                episode.created_at.timestamp_micros(),
                completed_at_ts,
            ],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    /// Retrieve an episode by ID.
    pub fn get_episode(&self, id: &str) -> Result<Option<Episode>, AresError> {
        let conn = self.store.get_conn()?;
        let result = conn.query_row(
            "SELECT id, mission_id, title, description, agents_involved, decisions_made,
                    outcome, score, cost, duration_secs, failures, lessons_learned,
                    tags, created_at, completed_at
             FROM episodes WHERE id = ?1",
            params![id],
            |row| Self::row_to_episode(row),
        );
        match result {
            Ok(ep) => Ok(Some(ep)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AresError::db(e)),
        }
    }

    /// Query episodes with filters.
    pub fn query_episodes(&self, query: &EpisodeQuery) -> Result<Vec<Episode>, AresError> {
        let conn = self.store.get_conn()?;
        let mut sql = String::from(
            "SELECT id, mission_id, title, description, agents_involved, decisions_made,
                    outcome, score, cost, duration_secs, failures, lessons_learned,
                    tags, created_at, completed_at
             FROM episodes WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut param_idx = 1;

        if let Some(ref outcome) = query.outcome {
            sql.push_str(&format!(" AND outcome = ?{}", param_idx));
            param_values.push(Box::new(outcome.as_str().to_string()));
            param_idx += 1;
        }
        if let Some(ref mid) = query.mission_id {
            sql.push_str(&format!(" AND mission_id = ?{}", param_idx));
            param_values.push(Box::new(mid.clone()));
            param_idx += 1;
        }
        if let Some(ref text) = query.search_text {
            sql.push_str(&format!(
                " AND (title LIKE ?{p} OR description LIKE ?{p})",
                p = param_idx
            ));
            param_values.push(Box::new(format!("%{}%", text)));
            param_idx += 1;
        }
        if let Some(min_score) = query.min_score {
            sql.push_str(&format!(" AND score >= ?{}", param_idx));
            param_values.push(Box::new(min_score));
            param_idx += 1;
        }
        let _ = param_idx; // suppress unused warning

        sql.push_str(" ORDER BY created_at DESC");

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        } else {
            sql.push_str(" LIMIT 100");
        }
        if let Some(offset) = query.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql).map_err(AresError::db)?;
        let rows = stmt
            .query_map(params_ref.as_slice(), |row| Self::row_to_episode(row))
            .map_err(AresError::db)?;

        let mut episodes = Vec::new();
        for row in rows {
            episodes.push(row.map_err(AresError::db)?);
        }

        // Post-filter by tags if specified (SQLite JSON matching is complex,
        // so we filter in application code for simplicity and correctness)
        if !query.tags.is_empty() {
            episodes.retain(|ep| query.tags.iter().any(|t| ep.tags.contains(t)));
        }

        Ok(episodes)
    }

    /// Find episodes similar to the given text by matching tags and description.
    pub fn find_similar(
        &self,
        description: &str,
        tags: &[String],
        limit: u32,
    ) -> Result<Vec<Episode>, AresError> {
        let conn = self.store.get_conn()?;
        // Use LIKE-based search on description and tags
        let search_pattern = format!("%{}%", description.split_whitespace().next().unwrap_or(""));
        let mut stmt = conn
            .prepare(
                "SELECT id, mission_id, title, description, agents_involved, decisions_made,
                        outcome, score, cost, duration_secs, failures, lessons_learned,
                        tags, created_at, completed_at
                 FROM episodes
                 WHERE description LIKE ?1 OR title LIKE ?1
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![search_pattern, limit], |row| {
                Self::row_to_episode(row)
            })
            .map_err(AresError::db)?;

        let mut episodes = Vec::new();
        for row in rows {
            episodes.push(row.map_err(AresError::db)?);
        }

        // Boost results that share tags
        if !tags.is_empty() {
            episodes.sort_by(|a, b| {
                let a_matches = tags.iter().filter(|t| a.tags.contains(t)).count();
                let b_matches = tags.iter().filter(|t| b.tags.contains(t)).count();
                b_matches.cmp(&a_matches)
            });
        }

        Ok(episodes)
    }

    /// Insert an episode event.
    pub fn insert_event(&self, event: &EpisodeEvent) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let metadata_str = serde_json::to_string(&event.metadata)
            .map_err(|e| AresError::Serialization(e.to_string()))?;

        conn.execute(
            "INSERT INTO episode_events (id, episode_id, event_type, description, agent_id, timestamp, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                event.id,
                event.episode_id,
                event.event_type.as_str(),
                event.description,
                event.agent_id,
                event.timestamp.timestamp_micros(),
                metadata_str,
            ],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    /// Get events for an episode, ordered by timestamp.
    pub fn get_events(&self, episode_id: &str) -> Result<Vec<EpisodeEvent>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, episode_id, event_type, description, agent_id, timestamp, metadata
                 FROM episode_events WHERE episode_id = ?1 ORDER BY timestamp ASC",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![episode_id], |row| {
                let ts: i64 = row.get(5)?;
                let metadata_str: String = row.get(6)?;
                Ok(EpisodeEvent {
                    id: row.get(0)?,
                    episode_id: row.get(1)?,
                    event_type: EpisodeEventType::from_str_val(&row.get::<_, String>(2)?),
                    description: row.get(3)?,
                    agent_id: row.get(4)?,
                    timestamp: Utc.timestamp_micros(ts).single().unwrap_or_else(Utc::now),
                    metadata: serde_json::from_str(&metadata_str).unwrap_or_default(),
                })
            })
            .map_err(AresError::db)?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row.map_err(AresError::db)?);
        }
        Ok(events)
    }

    /// Insert or replace an episode summary.
    pub fn upsert_summary(&self, summary: &EpisodeSummary) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let insights_json = serde_json::to_string(&summary.key_insights)
            .map_err(|e| AresError::Serialization(e.to_string()))?;

        conn.execute(
            "INSERT INTO episode_summaries (id, episode_id, summary_text, key_insights, compression_ratio, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(episode_id) DO UPDATE SET
               summary_text = excluded.summary_text,
               key_insights = excluded.key_insights,
               compression_ratio = excluded.compression_ratio,
               created_at = excluded.created_at",
            params![
                summary.id,
                summary.episode_id,
                summary.summary_text,
                insights_json,
                summary.compression_ratio,
                summary.created_at.timestamp_micros(),
            ],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    /// Get summary for an episode.
    pub fn get_summary(&self, episode_id: &str) -> Result<Option<EpisodeSummary>, AresError> {
        let conn = self.store.get_conn()?;
        let result = conn.query_row(
            "SELECT id, episode_id, summary_text, key_insights, compression_ratio, created_at
             FROM episode_summaries WHERE episode_id = ?1",
            params![episode_id],
            |row| {
                let ts: i64 = row.get(5)?;
                let insights_str: String = row.get(3)?;
                Ok(EpisodeSummary {
                    id: row.get(0)?,
                    episode_id: row.get(1)?,
                    summary_text: row.get(2)?,
                    key_insights: serde_json::from_str(&insights_str).unwrap_or_default(),
                    compression_ratio: row.get(4)?,
                    created_at: Utc.timestamp_micros(ts).single().unwrap_or_else(Utc::now),
                })
            },
        );
        match result {
            Ok(s) => Ok(Some(s)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AresError::db(e)),
        }
    }

    /// Count total episodes.
    pub fn count_episodes(&self) -> Result<u64, AresError> {
        let conn = self.store.get_conn()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM episodes", [], |row| row.get(0))
            .map_err(AresError::db)?;
        Ok(count as u64)
    }

    // ---- Internal helper ----

    fn row_to_episode(row: &rusqlite::Row<'_>) -> Result<Episode, rusqlite::Error> {
        let created_ts: i64 = row.get(13)?;
        let completed_ts: Option<i64> = row.get(14)?;
        let agents_str: String = row.get(4)?;
        let decisions_str: String = row.get(5)?;
        let failures_str: String = row.get(10)?;
        let lessons_str: String = row.get(11)?;
        let tags_str: String = row.get(12)?;

        Ok(Episode {
            id: row.get(0)?,
            mission_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            agents_involved: serde_json::from_str(&agents_str).unwrap_or_default(),
            decisions_made: serde_json::from_str(&decisions_str).unwrap_or_default(),
            outcome: EpisodeOutcome::from_str_val(&row.get::<_, String>(6)?),
            score: row.get(7)?,
            cost: row.get(8)?,
            duration_secs: row.get(9)?,
            failures: serde_json::from_str(&failures_str).unwrap_or_default(),
            lessons_learned: serde_json::from_str(&lessons_str).unwrap_or_default(),
            tags: serde_json::from_str(&tags_str).unwrap_or_default(),
            created_at: Utc
                .timestamp_micros(created_ts)
                .single()
                .unwrap_or_else(Utc::now),
            completed_at: completed_ts.and_then(|ts| Utc.timestamp_micros(ts).single()),
        })
    }
}

/// Helper to create a test episode with sensible defaults.
#[cfg(test)]
pub fn make_test_episode(id: &str, outcome: EpisodeOutcome) -> Episode {
    Episode {
        id: id.into(),
        mission_id: format!("mission_{}", id),
        title: format!("Test Episode {}", id),
        description: format!("Description for episode {}", id),
        agents_involved: vec!["agent_1".into()],
        decisions_made: vec!["decision_1".into()],
        outcome,
        score: 0.8,
        cost: 10.0,
        duration_secs: 120.0,
        failures: vec![],
        lessons_learned: vec!["learned something".into()],
        tags: vec!["test".into()],
        created_at: Utc::now(),
        completed_at: Some(Utc::now()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_store;
    use uuid::Uuid;

    #[test]
    fn insert_and_get_episode() {
        let (store, _dir) = test_store();
        let repo = EpisodeRepository::new(store);
        let ep = make_test_episode("ep_1", EpisodeOutcome::Success);

        repo.insert_episode(&ep).unwrap();
        let fetched = repo.get_episode("ep_1").unwrap().unwrap();
        assert_eq!(fetched.id, "ep_1");
        assert_eq!(fetched.outcome, EpisodeOutcome::Success);
        assert_eq!(fetched.agents_involved, vec!["agent_1".to_string()]);
    }

    #[test]
    fn get_nonexistent_episode_returns_none() {
        let (store, _dir) = test_store();
        let repo = EpisodeRepository::new(store);
        let result = repo.get_episode("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn query_episodes_by_outcome() {
        let (store, _dir) = test_store();
        let repo = EpisodeRepository::new(store);

        repo.insert_episode(&make_test_episode("ep_s", EpisodeOutcome::Success))
            .unwrap();
        repo.insert_episode(&make_test_episode("ep_f", EpisodeOutcome::Failure))
            .unwrap();

        let query = EpisodeQuery {
            outcome: Some(EpisodeOutcome::Success),
            ..Default::default()
        };
        let results = repo.query_episodes(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "ep_s");
    }

    #[test]
    fn query_episodes_by_search_text() {
        let (store, _dir) = test_store();
        let repo = EpisodeRepository::new(store);

        let mut ep = make_test_episode("ep_auth", EpisodeOutcome::Success);
        ep.title = "Build authentication system".into();
        ep.description = "Implemented JWT auth".into();
        repo.insert_episode(&ep).unwrap();

        let query = EpisodeQuery {
            search_text: Some("authentication".into()),
            ..Default::default()
        };
        let results = repo.query_episodes(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "ep_auth");
    }

    #[test]
    fn query_episodes_with_limit_and_offset() {
        let (store, _dir) = test_store();
        let repo = EpisodeRepository::new(store);

        for i in 0..5 {
            repo.insert_episode(&make_test_episode(
                &format!("ep_{}", i),
                EpisodeOutcome::Success,
            ))
            .unwrap();
        }

        let query = EpisodeQuery {
            limit: Some(2),
            offset: Some(1),
            ..Default::default()
        };
        let results = repo.query_episodes(&query).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn find_similar_episodes() {
        let (store, _dir) = test_store();
        let repo = EpisodeRepository::new(store);

        let mut ep1 = make_test_episode("ep_deploy1", EpisodeOutcome::Success);
        ep1.description = "Deploy service to production".into();
        ep1.tags = vec!["deploy".into(), "production".into()];
        repo.insert_episode(&ep1).unwrap();

        let mut ep2 = make_test_episode("ep_deploy2", EpisodeOutcome::Failure);
        ep2.description = "Deploy microservice failed".into();
        ep2.tags = vec!["deploy".into(), "microservice".into()];
        repo.insert_episode(&ep2).unwrap();

        let results = repo
            .find_similar("Deploy application", &["deploy".into()], 10)
            .unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn insert_and_get_events() {
        let (store, _dir) = test_store();
        let repo = EpisodeRepository::new(store);

        let ep = make_test_episode("ep_ev", EpisodeOutcome::Success);
        repo.insert_episode(&ep).unwrap();

        let ev1 = EpisodeEvent {
            id: Uuid::now_v7().to_string(),
            episode_id: "ep_ev".into(),
            event_type: EpisodeEventType::Action,
            description: "Started coding".into(),
            agent_id: Some("agent_1".into()),
            timestamp: Utc::now(),
            metadata: serde_json::json!({}),
        };
        let ev2 = EpisodeEvent {
            id: Uuid::now_v7().to_string(),
            episode_id: "ep_ev".into(),
            event_type: EpisodeEventType::Milestone,
            description: "Tests passing".into(),
            agent_id: None,
            timestamp: Utc::now(),
            metadata: serde_json::json!({"tests": 42}),
        };

        repo.insert_event(&ev1).unwrap();
        repo.insert_event(&ev2).unwrap();

        let events = repo.get_events("ep_ev").unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, EpisodeEventType::Action);
    }

    #[test]
    fn upsert_and_get_summary() {
        let (store, _dir) = test_store();
        let repo = EpisodeRepository::new(store);

        let ep = make_test_episode("ep_sum", EpisodeOutcome::Success);
        repo.insert_episode(&ep).unwrap();

        let summary = EpisodeSummary {
            id: "sum_1".into(),
            episode_id: "ep_sum".into(),
            summary_text: "Built and deployed auth system".into(),
            key_insights: vec!["JWT works well".into(), "Needs refresh logic".into()],
            compression_ratio: 0.25,
            created_at: Utc::now(),
        };
        repo.upsert_summary(&summary).unwrap();

        let fetched = repo.get_summary("ep_sum").unwrap().unwrap();
        assert_eq!(fetched.summary_text, "Built and deployed auth system");
        assert_eq!(fetched.key_insights.len(), 2);

        // Upsert again with updated text
        let updated = EpisodeSummary {
            id: "sum_1_v2".into(),
            summary_text: "Updated summary".into(),
            key_insights: vec!["Updated insight".into()],
            ..summary
        };
        repo.upsert_summary(&updated).unwrap();

        let fetched2 = repo.get_summary("ep_sum").unwrap().unwrap();
        assert_eq!(fetched2.summary_text, "Updated summary");
    }

    #[test]
    fn count_episodes() {
        let (store, _dir) = test_store();
        let repo = EpisodeRepository::new(store);

        assert_eq!(repo.count_episodes().unwrap(), 0);
        repo.insert_episode(&make_test_episode("ep_c1", EpisodeOutcome::Success))
            .unwrap();
        repo.insert_episode(&make_test_episode("ep_c2", EpisodeOutcome::Failure))
            .unwrap();
        assert_eq!(repo.count_episodes().unwrap(), 2);
    }

    #[test]
    fn query_with_min_score() {
        let (store, _dir) = test_store();
        let repo = EpisodeRepository::new(store);

        let mut ep_high = make_test_episode("ep_high", EpisodeOutcome::Success);
        ep_high.score = 0.95;
        repo.insert_episode(&ep_high).unwrap();

        let mut ep_low = make_test_episode("ep_low", EpisodeOutcome::Success);
        ep_low.score = 0.3;
        repo.insert_episode(&ep_low).unwrap();

        let query = EpisodeQuery {
            min_score: Some(0.5),
            ..Default::default()
        };
        let results = repo.query_episodes(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "ep_high");
    }

    #[test]
    fn query_episodes_by_tags() {
        let (store, _dir) = test_store();
        let repo = EpisodeRepository::new(store);

        let mut ep1 = make_test_episode("ep_tag1", EpisodeOutcome::Success);
        ep1.tags = vec!["auth".into(), "jwt".into()];
        repo.insert_episode(&ep1).unwrap();

        let mut ep2 = make_test_episode("ep_tag2", EpisodeOutcome::Success);
        ep2.tags = vec!["deploy".into()];
        repo.insert_episode(&ep2).unwrap();

        let query = EpisodeQuery {
            tags: vec!["auth".into()],
            ..Default::default()
        };
        let results = repo.query_episodes(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "ep_tag1");
    }
}
