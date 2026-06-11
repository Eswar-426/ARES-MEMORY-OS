//! Memory Advisor — provides memory-augmented planning recommendations.
//!
//! Before generating a plan, the planner queries the memory advisor for:
//! - Past similar missions and their outcomes
//! - Known failure patterns to avoid
//! - Relevant principles to follow
//! - Recommended strategies from past successes

use ares_core::AresError;
use ares_memory_intelligence::episodic::engine::EpisodicMemoryEngine;
use ares_memory_intelligence::episodic::models::{EpisodeOutcome, EpisodeQuery};
use ares_memory_intelligence::experience::engine::ExperienceLearningEngine;
use ares_store::db::Store;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Advice from the memory system for a planning session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningAdvice {
    /// Past similar missions ranked by relevance.
    pub similar_missions: Vec<MissionPrecedent>,
    /// Known failure patterns to watch for.
    pub failure_warnings: Vec<String>,
    /// Principles that apply to this type of mission.
    pub applicable_principles: Vec<String>,
    /// Recommended approaches from past successes.
    pub recommended_approaches: Vec<String>,
    /// Overall confidence in the advice (0.0..1.0).
    pub confidence: f64,
}

/// A past mission relevant to the current planning context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionPrecedent {
    pub episode_id: String,
    pub title: String,
    pub outcome: String,
    pub relevance_score: f64,
    pub lessons: Vec<String>,
}

/// Memory advisor for the planner.
pub struct MemoryAdvisor {
    episodic: EpisodicMemoryEngine,
    experience: ExperienceLearningEngine,
}

impl MemoryAdvisor {
    pub fn new(store: Store) -> Self {
        Self {
            episodic: EpisodicMemoryEngine::new(store.clone()),
            experience: ExperienceLearningEngine::new(store),
        }
    }

    /// Get planning advice for a mission description.
    pub fn advise(
        &self,
        mission_description: &str,
        tags: &[String],
    ) -> Result<PlanningAdvice, AresError> {
        debug!("Memory advisor: generating advice for mission");

        // 1. Find similar past missions
        let similar = self.episodic.find_similar(mission_description, tags, 10)?;
        let ranked = self.episodic.rank_episodes(&similar);

        let similar_missions: Vec<MissionPrecedent> = ranked
            .iter()
            .take(5)
            .map(|r| MissionPrecedent {
                episode_id: r.episode.id.clone(),
                title: r.episode.title.clone(),
                outcome: r.episode.outcome.as_str().into(),
                relevance_score: r.relevance_score,
                lessons: r.episode.lessons_learned.clone(),
            })
            .collect();

        // 2. Retrieve failure patterns
        let failure_query = EpisodeQuery {
            outcome: Some(EpisodeOutcome::Failure),
            search_text: Some(mission_description.into()),
            limit: Some(5),
            ..Default::default()
        };
        let failure_episodes = self.episodic.query_episodes(&failure_query)?;
        let failure_warnings: Vec<String> = failure_episodes
            .iter()
            .flat_map(|ep| {
                let mut warnings = ep.failures.clone();
                warnings.extend(ep.lessons_learned.iter().cloned());
                warnings
            })
            .collect();

        // 3. Get applicable principles
        let principles = self.experience.list_principles(None)?;
        let applicable_principles: Vec<String> = principles
            .iter()
            .filter(|p| {
                let lower = mission_description.to_lowercase();
                p.title.to_lowercase().contains(&lower)
                    || p.description.to_lowercase().contains(&lower)
                    || p.domain == "general"
            })
            .map(|p| format!("{}: {}", p.title, p.description))
            .collect();

        // 4. Extract approaches from successful missions
        let success_query = EpisodeQuery {
            outcome: Some(EpisodeOutcome::Success),
            search_text: Some(mission_description.into()),
            limit: Some(5),
            ..Default::default()
        };
        let success_episodes = self.episodic.query_episodes(&success_query)?;
        let recommended_approaches: Vec<String> = success_episodes
            .iter()
            .flat_map(|ep| ep.decisions_made.clone())
            .collect();

        // Calculate overall confidence based on how much evidence we have
        let evidence_count = similar_missions.len()
            + failure_warnings.len()
            + applicable_principles.len()
            + recommended_approaches.len();
        let confidence = if evidence_count == 0 {
            0.0
        } else {
            (evidence_count as f64 / 20.0).min(1.0)
        };

        Ok(PlanningAdvice {
            similar_missions,
            failure_warnings,
            applicable_principles,
            recommended_approaches,
            confidence,
        })
    }

    /// Quick check: have we done something similar before?
    pub fn has_precedent(&self, description: &str) -> Result<bool, AresError> {
        let similar = self.episodic.find_similar(description, &[], 1)?;
        Ok(!similar.is_empty())
    }

    /// Get failure rate for missions matching this description.
    pub fn failure_rate(&self, description: &str) -> Result<f64, AresError> {
        let similar = self.episodic.find_similar(description, &[], 20)?;
        if similar.is_empty() {
            return Ok(0.0);
        }
        let failures = similar
            .iter()
            .filter(|ep| ep.outcome == EpisodeOutcome::Failure)
            .count();
        Ok(failures as f64 / similar.len() as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ares_memory_intelligence::episodic::models::{Episode, EpisodeOutcome};
    use ares_memory_intelligence::test_utils::test_store;
    use chrono::Utc;

    fn make_test_episode(id: &str, outcome: EpisodeOutcome) -> Episode {
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

    fn make_advisor() -> (MemoryAdvisor, Store, tempfile::TempDir) {
        let (store, dir) = test_store();
        let advisor = MemoryAdvisor::new(store.clone());
        (advisor, store, dir)
    }

    #[test]
    fn advise_empty_db() {
        let (advisor, _, _dir) = make_advisor();
        let advice = advisor.advise("Deploy service", &[]).unwrap();
        assert!(advice.similar_missions.is_empty());
        assert_eq!(advice.confidence, 0.0);
    }

    #[test]
    fn advise_with_past_missions() {
        let (advisor, store, _dir) = make_advisor();
        let episodic = EpisodicMemoryEngine::new(store);

        let mut ep = make_test_episode("ep_past", EpisodeOutcome::Success);
        ep.title = "Deploy microservice to production".into();
        ep.description = "Deployed service using k8s".into();
        ep.decisions_made = vec!["Use rolling deployment".into()];
        ep.lessons_learned = vec!["Check health first".into()];
        episodic.store_episode(&ep).unwrap();

        let advice = advisor
            .advise("Deploy application to production", &[])
            .unwrap();
        assert!(!advice.similar_missions.is_empty());
        assert!(advice.confidence > 0.0);
    }

    #[test]
    fn has_precedent_empty() {
        let (advisor, _, _dir) = make_advisor();
        assert!(!advisor.has_precedent("something new").unwrap());
    }

    #[test]
    fn has_precedent_found() {
        let (advisor, store, _dir) = make_advisor();
        let episodic = EpisodicMemoryEngine::new(store);
        let mut ep = make_test_episode("ep_prec", EpisodeOutcome::Success);
        ep.description = "Implemented caching layer".into();
        episodic.store_episode(&ep).unwrap();

        assert!(advisor.has_precedent("Implemented caching").unwrap());
    }

    #[test]
    fn failure_rate_empty() {
        let (advisor, _, _dir) = make_advisor();
        let rate = advisor.failure_rate("deploy").unwrap();
        assert!((rate - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn failure_rate_calculation() {
        let (advisor, store, _dir) = make_advisor();
        let episodic = EpisodicMemoryEngine::new(store);

        for i in 0..4 {
            let mut ep = make_test_episode(
                &format!("ep_rate_{}", i),
                if i < 3 {
                    EpisodeOutcome::Success
                } else {
                    EpisodeOutcome::Failure
                },
            );
            ep.description = "Deploy service task".into();
            episodic.store_episode(&ep).unwrap();
        }

        let rate = advisor.failure_rate("Deploy").unwrap();
        // 1 failure out of 4
        assert!(rate > 0.0 && rate <= 1.0);
    }

    #[test]
    fn planning_advice_serialization() {
        let advice = PlanningAdvice {
            similar_missions: vec![MissionPrecedent {
                episode_id: "ep_1".into(),
                title: "Past mission".into(),
                outcome: "success".into(),
                relevance_score: 0.9,
                lessons: vec!["lesson 1".into()],
            }],
            failure_warnings: vec!["Watch for timeouts".into()],
            applicable_principles: vec!["Always validate first".into()],
            recommended_approaches: vec!["Use rolling deploy".into()],
            confidence: 0.75,
        };
        let json = serde_json::to_string(&advice).unwrap();
        let back: PlanningAdvice = serde_json::from_str(&json).unwrap();
        assert_eq!(back.similar_missions.len(), 1);
        assert!((back.confidence - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn mission_precedent_serialization() {
        let mp = MissionPrecedent {
            episode_id: "ep_1".into(),
            title: "Deploy".into(),
            outcome: "success".into(),
            relevance_score: 0.8,
            lessons: vec![],
        };
        let json = serde_json::to_string(&mp).unwrap();
        let back: MissionPrecedent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.episode_id, "ep_1");
    }
}
