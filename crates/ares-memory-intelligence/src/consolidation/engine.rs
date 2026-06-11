use super::models::*;
use super::repository::ConsolidationRepository;
use crate::compression::engine::CompressionEngine;
use crate::compression::models::CompressionConfig;
use crate::episodic::models::{Episode, EpisodeQuery};
use crate::episodic::repository::EpisodeRepository;
use ares_core::AresError;
use ares_store::db::Store;
use chrono::Utc;
use std::collections::HashMap;
use tracing::debug;
use uuid::Uuid;

/// Engine for human-like long-term memory formation.
/// Merges duplicates, clusters related memories, detects patterns, generates summaries.
pub struct ConsolidationEngine {
    episode_repo: EpisodeRepository,
    cluster_repo: ConsolidationRepository,
    compressor: CompressionEngine,
    config: ConsolidationConfig,
}

impl ConsolidationEngine {
    pub fn new(store: Store, config: ConsolidationConfig) -> Self {
        Self {
            episode_repo: EpisodeRepository::new(store.clone()),
            cluster_repo: ConsolidationRepository::new(store),
            compressor: CompressionEngine::new(CompressionConfig {
                dedup_threshold: config.merge_threshold,
                ..Default::default()
            }),
            config,
        }
    }

    pub fn with_defaults(store: Store) -> Self {
        Self::new(store, ConsolidationConfig::default())
    }

    /// Detect recurring patterns across episodes.
    pub fn detect_patterns(&self, episodes: &[Episode]) -> Vec<RecurringPattern> {
        // Track tag frequency across episodes
        let mut tag_episodes: HashMap<String, Vec<String>> = HashMap::new();
        for ep in episodes {
            for tag in &ep.tags {
                tag_episodes
                    .entry(tag.clone())
                    .or_default()
                    .push(ep.id.clone());
            }
        }

        // Track lesson frequency
        let mut lesson_episodes: HashMap<String, Vec<String>> = HashMap::new();
        for ep in episodes {
            for lesson in &ep.lessons_learned {
                let key = lesson.to_lowercase();
                lesson_episodes.entry(key).or_default().push(ep.id.clone());
            }
        }

        let mut patterns = Vec::new();

        // Tag-based patterns
        for (tag, ep_ids) in &tag_episodes {
            if ep_ids.len() >= self.config.min_cluster_size {
                patterns.push(RecurringPattern {
                    description: format!("Recurring topic: {}", tag),
                    frequency: ep_ids.len() as u32,
                    episode_ids: ep_ids.clone(),
                    confidence: (ep_ids.len() as f64 / episodes.len().max(1) as f64).min(1.0),
                });
            }
        }

        // Lesson-based patterns
        for (lesson, ep_ids) in &lesson_episodes {
            if ep_ids.len() >= self.config.min_cluster_size {
                patterns.push(RecurringPattern {
                    description: format!("Recurring lesson: {}", lesson),
                    frequency: ep_ids.len() as u32,
                    episode_ids: ep_ids.clone(),
                    confidence: (ep_ids.len() as f64 / episodes.len().max(1) as f64).min(1.0),
                });
            }
        }

        patterns.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        patterns
    }

    /// Cluster related episodes by tag overlap.
    pub fn cluster_related(&self, episodes: &[Episode]) -> Result<Vec<MemoryCluster>, AresError> {
        let items: Vec<(String, String)> = episodes
            .iter()
            .map(|ep| {
                let text = format!("{} {} {}", ep.title, ep.description, ep.tags.join(" "));
                (ep.id.clone(), text)
            })
            .collect();

        let compression_clusters = self.compressor.cluster(&items);
        let now = Utc::now().timestamp_micros();
        let mut clusters = Vec::new();

        for cc in compression_clusters {
            if cc.member_ids.len() >= self.config.min_cluster_size {
                // Collect common tags
                let member_episodes: Vec<&Episode> = episodes
                    .iter()
                    .filter(|ep| cc.member_ids.contains(&ep.id))
                    .collect();

                let mut tag_freq: HashMap<&str, usize> = HashMap::new();
                for ep in &member_episodes {
                    for tag in &ep.tags {
                        *tag_freq.entry(tag.as_str()).or_insert(0) += 1;
                    }
                }
                let mut tags: Vec<(&str, usize)> = tag_freq.into_iter().collect();
                tags.sort_by(|a, b| b.1.cmp(&a.1));
                let centroid_tags: Vec<String> =
                    tags.iter().take(5).map(|(t, _)| t.to_string()).collect();

                let cluster = MemoryCluster {
                    id: Uuid::now_v7().to_string(),
                    name: format!(
                        "Cluster: {}",
                        centroid_tags.first().unwrap_or(&"misc".to_string())
                    ),
                    description: cc.summary.clone(),
                    cluster_type: ClusterType::Topic,
                    member_count: cc.member_ids.len() as u32,
                    centroid_tags,
                    summary: cc.summary,
                    created_at: now,
                    updated_at: now,
                };

                self.cluster_repo.insert_cluster(&cluster)?;

                // Add memberships
                for ep_id in &cc.member_ids {
                    let membership = ClusterMembership {
                        cluster_id: cluster.id.clone(),
                        episode_id: ep_id.clone(),
                        similarity: 0.8,
                        added_at: now,
                    };
                    // Ignore FK errors if episode doesn't exist in DB
                    let _ = self.cluster_repo.add_membership(&membership);
                }

                clusters.push(cluster);
            }
        }

        debug!(clusters = clusters.len(), "Formed memory clusters");
        Ok(clusters)
    }

    /// Archive low-value events (episodes with low scores and old age).
    pub fn archive_low_value(&self, episodes: &[Episode]) -> Vec<String> {
        let cutoff = Utc::now() - chrono::Duration::days(self.config.archive_age_days as i64);
        episodes
            .iter()
            .filter(|ep| ep.score < self.config.archive_min_score && ep.created_at < cutoff)
            .map(|ep| ep.id.clone())
            .collect()
    }

    /// Run a full consolidation cycle on all episodes.
    pub fn run_consolidation_cycle(&self) -> Result<ConsolidationResult, AresError> {
        debug!("Starting consolidation cycle");

        // Fetch all episodes
        let query = EpisodeQuery {
            limit: Some(1000),
            ..Default::default()
        };
        let episodes = self.episode_repo.query_episodes(&query)?;

        if episodes.is_empty() {
            return Ok(ConsolidationResult {
                duplicates_merged: 0,
                clusters_formed: 0,
                patterns_detected: 0,
                summaries_generated: 0,
                events_archived: 0,
            });
        }

        // Detect patterns
        let patterns = self.detect_patterns(&episodes);

        // Cluster related
        let clusters = self.cluster_related(&episodes)?;

        // Archive low-value
        let archived = self.archive_low_value(&episodes);

        Ok(ConsolidationResult {
            duplicates_merged: 0, // Dedup is handled by compression engine
            clusters_formed: clusters.len() as u32,
            patterns_detected: patterns.len() as u32,
            summaries_generated: clusters.len() as u32,
            events_archived: archived.len() as u32,
        })
    }

    /// List existing clusters.
    pub fn list_clusters(&self) -> Result<Vec<MemoryCluster>, AresError> {
        self.cluster_repo.list_clusters()
    }

    /// Count clusters.
    pub fn count_clusters(&self) -> Result<u64, AresError> {
        self.cluster_repo.count_clusters()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episodic::models::EpisodeOutcome;
    use crate::episodic::repository::make_test_episode;
    use crate::test_utils::test_store;

    fn make_engine() -> (ConsolidationEngine, tempfile::TempDir) {
        let (store, dir) = test_store();
        (ConsolidationEngine::with_defaults(store), dir)
    }

    #[test]
    fn detect_patterns_empty() {
        let (engine, _dir) = make_engine();
        let patterns = engine.detect_patterns(&[]);
        assert!(patterns.is_empty());
    }

    #[test]
    fn detect_patterns_finds_recurring_tags() {
        let (engine, _dir) = make_engine();
        let mut episodes = Vec::new();
        for i in 0..5 {
            let mut ep = make_test_episode(&format!("ep_{}", i), EpisodeOutcome::Success);
            ep.tags = vec!["deploy".into(), "production".into()];
            episodes.push(ep);
        }

        let patterns = engine.detect_patterns(&episodes);
        assert!(!patterns.is_empty());
        assert!(patterns.iter().any(|p| p.description.contains("deploy")));
    }

    #[test]
    fn detect_patterns_finds_recurring_lessons() {
        let (engine, _dir) = make_engine();
        let mut episodes = Vec::new();
        for i in 0..4 {
            let mut ep = make_test_episode(&format!("ep_{}", i), EpisodeOutcome::Failure);
            ep.lessons_learned = vec!["Validate environment first".into()];
            episodes.push(ep);
        }

        let patterns = engine.detect_patterns(&episodes);
        assert!(!patterns.is_empty());
    }

    #[test]
    fn archive_low_value_filters_correctly() {
        let (engine, _dir) = make_engine();
        let mut old_low = make_test_episode("ep_old", EpisodeOutcome::Unknown);
        old_low.score = 0.1;
        old_low.created_at = Utc::now() - chrono::Duration::days(120);

        let recent = make_test_episode("ep_recent", EpisodeOutcome::Success);

        let archived = engine.archive_low_value(&[old_low, recent]);
        assert_eq!(archived.len(), 1);
        assert_eq!(archived[0], "ep_old");
    }

    #[test]
    fn run_consolidation_cycle_empty_db() {
        let (engine, _dir) = make_engine();
        let result = engine.run_consolidation_cycle().unwrap();
        assert_eq!(result.clusters_formed, 0);
        assert_eq!(result.patterns_detected, 0);
    }

    #[test]
    fn cluster_related_empty() {
        let (engine, _dir) = make_engine();
        let clusters = engine.cluster_related(&[]).unwrap();
        assert!(clusters.is_empty());
    }

    #[test]
    fn count_clusters_initially_zero() {
        let (engine, _dir) = make_engine();
        assert_eq!(engine.count_clusters().unwrap(), 0);
    }
}
