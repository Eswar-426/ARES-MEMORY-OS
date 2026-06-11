//! Memory bridge — connects agent runtime reflection/learning to the
//! memory intelligence layer.
//!
//! After each mission, the bridge:
//! 1. Converts the reflection into an episode
//! 2. Extracts semantic concepts
//! 3. Records decisions
//! 4. Feeds the experience learning pipeline

use ares_core::AresError;
use ares_memory_intelligence::episodic::engine::EpisodicMemoryEngine;
use ares_memory_intelligence::episodic::models::{Episode, EpisodeOutcome};
use ares_memory_intelligence::experience::engine::ExperienceLearningEngine;
use ares_memory_intelligence::retrieval::engine::RetrievalEngine;
use ares_memory_intelligence::retrieval::models::{
    RetrievalQueryType, RetrievalRequest, RetrievalResponse,
};
use ares_memory_intelligence::semantic::engine::SemanticMemoryEngine;
use ares_store::db::Store;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use uuid::Uuid;

/// Input for the memory bridge — data from a completed mission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionMemoryInput {
    pub mission_id: String,
    pub title: String,
    pub description: String,
    pub agents_involved: Vec<String>,
    pub decisions_made: Vec<String>,
    pub outcome: String, // "success", "failure", etc.
    pub score: f64,
    pub cost: f64,
    pub duration_secs: f64,
    pub failures: Vec<String>,
    pub lessons_learned: Vec<String>,
    pub tags: Vec<String>,
}

/// Result from processing a mission through the memory bridge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBridgeResult {
    pub episode_id: String,
    pub semantic_memory_ids: Vec<String>,
    pub experience_created: bool,
    pub lessons_count: usize,
}

/// The memory bridge — ties agent runtime lifecycle events to persistent memory.
pub struct MemoryBridge {
    episodic: EpisodicMemoryEngine,
    semantic: SemanticMemoryEngine,
    experience: ExperienceLearningEngine,
    retrieval: RetrievalEngine,
}

impl MemoryBridge {
    pub fn new(store: Store) -> Self {
        Self {
            episodic: EpisodicMemoryEngine::new(store.clone()),
            semantic: SemanticMemoryEngine::new(store.clone()),
            experience: ExperienceLearningEngine::new(store.clone()),
            retrieval: RetrievalEngine::new(store),
        }
    }

    /// Process a completed mission: store episode, extract semantics, record experience.
    pub fn process_mission(
        &self,
        input: &MissionMemoryInput,
    ) -> Result<MemoryBridgeResult, AresError> {
        let episode_id = Uuid::now_v7().to_string();
        info!(
            episode_id = %episode_id,
            mission_id = %input.mission_id,
            "Processing mission into memory"
        );

        // 1. Create and store episode
        let episode = Episode {
            id: episode_id.clone(),
            mission_id: input.mission_id.clone(),
            title: input.title.clone(),
            description: input.description.clone(),
            agents_involved: input.agents_involved.clone(),
            decisions_made: input.decisions_made.clone(),
            outcome: EpisodeOutcome::from_str_val(&input.outcome),
            score: input.score,
            cost: input.cost,
            duration_secs: input.duration_secs,
            failures: input.failures.clone(),
            lessons_learned: input.lessons_learned.clone(),
            tags: input.tags.clone(),
            created_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };
        self.episodic.store_episode(&episode)?;
        debug!(episode_id = %episode_id, "Stored episode");

        // 2. Extract semantic concepts from description + lessons
        let text = format!(
            "{} {} {}",
            input.description,
            input.lessons_learned.join(". "),
            input.failures.join(". "),
        );
        let extraction = self.semantic.extract_concepts(&text, Some(&episode_id))?;
        let semantic_ids = self
            .semantic
            .store_extraction(&extraction, Some(&episode_id))?;
        debug!(count = semantic_ids.len(), "Extracted semantic memories");

        // 3. Feed experience learning pipeline
        let exp = self.experience.record_experience_from_episode(&episode)?;
        debug!(experience_id = %exp.id, "Recorded experience");

        Ok(MemoryBridgeResult {
            episode_id,
            semantic_memory_ids: semantic_ids,
            experience_created: true,
            lessons_count: input.lessons_learned.len(),
        })
    }

    /// Retrieve relevant memories before starting a new mission.
    pub fn pre_mission_retrieval(
        &self,
        mission_description: &str,
        tags: &[String],
    ) -> Result<RetrievalResponse, AresError> {
        let request = RetrievalRequest {
            query_text: mission_description.into(),
            query_type: RetrievalQueryType::SimilarMission,
            tags: tags.to_vec(),
            max_results: 10,
            min_confidence: 0.3,
        };
        self.retrieval.retrieve(&request)
    }

    /// Retrieve past failures relevant to a mission.
    pub fn retrieve_failures(
        &self,
        mission_description: &str,
    ) -> Result<RetrievalResponse, AresError> {
        let request = RetrievalRequest {
            query_text: mission_description.into(),
            query_type: RetrievalQueryType::FailureSearch,
            max_results: 5,
            ..Default::default()
        };
        self.retrieval.retrieve(&request)
    }

    /// Get count of stored episodes.
    pub fn episode_count(&self) -> Result<u64, AresError> {
        self.episodic.count()
    }

    /// Get count of stored semantic memories.
    pub fn semantic_count(&self) -> Result<u64, AresError> {
        self.semantic.count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ares_memory_intelligence::test_utils::test_store;

    fn make_bridge() -> (MemoryBridge, tempfile::TempDir) {
        let (store, dir) = test_store();
        (MemoryBridge::new(store), dir)
    }

    fn make_input() -> MissionMemoryInput {
        MissionMemoryInput {
            mission_id: "m_test".into(),
            title: "Build authentication system".into(),
            description: "Implemented JWT-based authentication with OAuth integration".into(),
            agents_involved: vec!["coder_agent".into(), "reviewer_agent".into()],
            decisions_made: vec!["Chose JWT over sessions".into()],
            outcome: "success".into(),
            score: 0.9,
            cost: 15.0,
            duration_secs: 300.0,
            failures: vec![],
            lessons_learned: vec!["JWT requires token refresh logic".into()],
            tags: vec!["auth".into(), "jwt".into()],
        }
    }

    #[test]
    fn process_mission_full_pipeline() {
        let (bridge, _dir) = make_bridge();
        let input = make_input();

        let result = bridge.process_mission(&input).unwrap();
        assert!(!result.episode_id.is_empty());
        assert!(result.experience_created);
        assert_eq!(result.lessons_count, 1);
        // Semantic extraction should produce results from "JWT", "authentication", etc.
        assert!(!result.semantic_memory_ids.is_empty());

        assert_eq!(bridge.episode_count().unwrap(), 1);
        assert!(bridge.semantic_count().unwrap() > 0);
    }

    #[test]
    fn process_failure_mission() {
        let (bridge, _dir) = make_bridge();
        let mut input = make_input();
        input.outcome = "failure".into();
        input.failures = vec!["Timeout during deployment".into()];
        input.lessons_learned = vec!["Check health endpoint first".into()];

        let result = bridge.process_mission(&input).unwrap();
        assert!(!result.episode_id.is_empty());
        assert!(result.experience_created);
    }

    #[test]
    fn pre_mission_retrieval_empty_db() {
        let (bridge, _dir) = make_bridge();
        let response = bridge
            .pre_mission_retrieval("Deploy microservice", &["deploy".into()])
            .unwrap();
        assert!(response.results.is_empty());
    }

    #[test]
    fn pre_mission_retrieval_finds_past_episodes() {
        let (bridge, _dir) = make_bridge();

        // Process a mission first
        let mut input = make_input();
        input.title = "Deploy microservice to production".into();
        input.description = "Deployed service to kubernetes cluster".into();
        input.tags = vec!["deploy".into(), "k8s".into()];
        bridge.process_mission(&input).unwrap();

        // Now retrieve relevant memories — use a term that's a substring
        // of the stored title, since retrieval uses LIKE-based search.
        let response = bridge
            .pre_mission_retrieval("Deploy", &["deploy".into()])
            .unwrap();
        assert!(!response.results.is_empty());
    }

    #[test]
    fn retrieve_failures_empty_db() {
        let (bridge, _dir) = make_bridge();
        let response = bridge.retrieve_failures("deploy issues").unwrap();
        assert!(response.results.is_empty());
    }

    #[test]
    fn multiple_missions_accumulate() {
        let (bridge, _dir) = make_bridge();

        for i in 0..3 {
            let mut input = make_input();
            input.mission_id = format!("m_{}", i);
            input.title = format!("Mission {}", i);
            bridge.process_mission(&input).unwrap();
        }

        assert_eq!(bridge.episode_count().unwrap(), 3);
    }

    #[test]
    fn mission_memory_input_serialization() {
        let input = make_input();
        let json = serde_json::to_string(&input).unwrap();
        let back: MissionMemoryInput = serde_json::from_str(&json).unwrap();
        assert_eq!(back.title, "Build authentication system");
    }

    #[test]
    fn memory_bridge_result_serialization() {
        let result = MemoryBridgeResult {
            episode_id: "ep_1".into(),
            semantic_memory_ids: vec!["sm_1".into()],
            experience_created: true,
            lessons_count: 2,
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: MemoryBridgeResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.episode_id, "ep_1");
    }
}
