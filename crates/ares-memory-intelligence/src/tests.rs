//! Integration tests that exercise full pipelines across modules.

use crate::compression::engine::CompressionEngine;
use crate::consolidation::engine::ConsolidationEngine;
use crate::decision_intelligence::engine::DecisionIntelligenceEngine;
use crate::decision_intelligence::models::{DecisionAlternative, DecisionType};
use crate::episodic::engine::EpisodicMemoryEngine;
use crate::episodic::models::{EpisodeEventType, EpisodeOutcome};
use crate::episodic::repository::make_test_episode;
use crate::evolution::engine::KnowledgeEvolutionEngine;
use crate::experience::engine::ExperienceLearningEngine;
use crate::retrieval::engine::RetrievalEngine;
use crate::retrieval::models::{RetrievalQueryType, RetrievalRequest};
use crate::semantic::engine::SemanticMemoryEngine;
use crate::test_utils::test_store;
use chrono::Utc;
use uuid::Uuid;

/// Full lifecycle: episode → semantic extraction → evolution → retrieval.
#[test]
fn full_memory_lifecycle() {
    let (store, _dir) = test_store();

    // 1. Store an episode
    let episodic = EpisodicMemoryEngine::new(store.clone());
    let ep = make_test_episode("ep_lifecycle", EpisodeOutcome::Success);
    episodic.store_episode(&ep).unwrap();
    assert_eq!(episodic.count().unwrap(), 1);

    // 2. Extract concepts
    let semantic = SemanticMemoryEngine::new(store.clone());
    let extraction = semantic
        .extract_concepts(
            "Deploy microservice to Kubernetes using Docker",
            Some("ep_lifecycle"),
        )
        .unwrap();
    assert!(!extraction.entities.is_empty());

    let ids = semantic
        .store_extraction(&extraction, Some("ep_lifecycle"))
        .unwrap();
    assert!(!ids.is_empty());
    assert!(semantic.count().unwrap() > 0);

    // 3. Evolve knowledge
    let evolution = KnowledgeEvolutionEngine::with_defaults(store.clone());
    let entry = evolution
        .reinforce(&ids[0], 0.05, "Confirmed by episode", Some("ep_lifecycle"))
        .unwrap();
    assert!(entry.new_confidence > entry.old_confidence);

    // 4. Retrieve
    let retrieval = RetrievalEngine::new(store.clone());
    let request = RetrievalRequest {
        query_text: "Deploy".into(),
        query_type: RetrievalQueryType::General,
        max_results: 10,
        ..Default::default()
    };
    let response = retrieval.retrieve(&request).unwrap();
    // Should find the episode
    assert!(!response.results.is_empty());
}

/// Episode → Experience → Lesson pipeline.
#[test]
fn episode_to_experience_pipeline() {
    let (store, _dir) = test_store();

    let episodic = EpisodicMemoryEngine::new(store.clone());
    let experience = ExperienceLearningEngine::new(store.clone());

    // Store episodes
    let ep1 = make_test_episode("ep_exp_1", EpisodeOutcome::Failure);
    let ep2 = make_test_episode("ep_exp_2", EpisodeOutcome::Success);
    episodic.store_episode(&ep1).unwrap();
    episodic.store_episode(&ep2).unwrap();

    // Convert to experiences
    let exp1 = experience.record_experience_from_episode(&ep1).unwrap();
    let exp2 = experience.record_experience_from_episode(&ep2).unwrap();

    assert_eq!(
        exp1.experience_type,
        crate::experience::models::ExperienceType::FailurePattern
    );
    assert_eq!(
        exp2.experience_type,
        crate::experience::models::ExperienceType::SuccessPattern
    );
}

/// Decision intelligence: record, query, set outcome, explain.
#[test]
fn decision_intelligence_full_cycle() {
    let (store, _dir) = test_store();
    let engine = DecisionIntelligenceEngine::new(store);

    let rec = engine
        .record_decision(
            Some("ep_di"),
            DecisionType::Technical,
            "Which serialization format?",
            "JSON",
            vec![
                DecisionAlternative {
                    option: "MessagePack".into(),
                    reason_rejected: "Less readable".into(),
                },
                DecisionAlternative {
                    option: "Protobuf".into(),
                    reason_rejected: "Requires schema files".into(),
                },
            ],
            "JSON is human-readable and widely supported",
            0.85,
        )
        .unwrap();

    // Check alternatives
    let alts = engine.get_alternatives(&rec.id).unwrap();
    assert_eq!(alts.len(), 2);

    // Set outcome
    engine
        .set_outcome(
            &rec.id,
            crate::decision_intelligence::models::DecisionOutcomeType::Positive,
        )
        .unwrap();

    // Explain
    let explanation = engine.explain_decision(&rec.id).unwrap();
    assert_eq!(explanation.decision.chosen_option, "JSON");
}

/// Consolidation: detect patterns across episodes.
#[test]
fn consolidation_pattern_detection() {
    let (store, _dir) = test_store();
    let episodic = EpisodicMemoryEngine::new(store.clone());
    let consolidation = ConsolidationEngine::with_defaults(store);

    // Create episodes with shared tags
    let mut episodes = Vec::new();
    for i in 0..5 {
        let mut ep = make_test_episode(&format!("ep_consol_{}", i), EpisodeOutcome::Success);
        ep.tags = vec!["deploy".into(), "k8s".into()];
        ep.lessons_learned = vec!["Check health endpoint".into()];
        episodic.store_episode(&ep).unwrap();
        episodes.push(ep);
    }

    let patterns = consolidation.detect_patterns(&episodes);
    assert!(!patterns.is_empty());
    assert!(patterns.iter().any(|p| p.description.contains("deploy")));
}

/// Compression: summarize, deduplicate, cluster.
#[test]
fn compression_pipeline() {
    let engine = CompressionEngine::with_defaults();

    let items = vec![
        ("1".into(), "Authentication system using JWT tokens".into()),
        ("2".into(), "Build REST API endpoint".into()),
        ("3".into(), "Deploy service to production".into()),
        (
            "4".into(),
            "Authentication mechanism with JWT validation".into(),
        ),
    ];

    let result = engine.compress(&items);
    assert_eq!(result.stats.input_count, 4);
    assert!(result.stats.output_count <= 4);
}

/// Retrieval: search failures and successes independently.
#[test]
fn retrieval_failure_vs_success() {
    let (store, _dir) = test_store();
    let episodic = EpisodicMemoryEngine::new(store.clone());
    let retrieval = RetrievalEngine::new(store);

    episodic
        .store_episode(&make_test_episode("ep_succ_r", EpisodeOutcome::Success))
        .unwrap();
    episodic
        .store_episode(&make_test_episode("ep_fail_r", EpisodeOutcome::Failure))
        .unwrap();

    // Search failures
    let fail_req = RetrievalRequest {
        query_text: "".into(),
        query_type: RetrievalQueryType::FailureSearch,
        max_results: 10,
        ..Default::default()
    };
    let fail_resp = retrieval.retrieve(&fail_req).unwrap();

    // Search successes
    let succ_req = RetrievalRequest {
        query_text: "".into(),
        query_type: RetrievalQueryType::SuccessSearch,
        max_results: 10,
        ..Default::default()
    };
    let succ_resp = retrieval.retrieve(&succ_req).unwrap();

    // Both should have results from their respective categories
    assert!(!fail_resp.results.is_empty());
    assert!(!succ_resp.results.is_empty());
}

/// Episode summarization with events.
#[test]
fn episode_summarization() {
    let (store, _dir) = test_store();
    let episodic = EpisodicMemoryEngine::new(store);

    let ep = make_test_episode("ep_summary_test", EpisodeOutcome::Success);
    episodic.store_episode(&ep).unwrap();

    // Add events
    let events = vec![
        crate::episodic::models::EpisodeEvent {
            id: Uuid::now_v7().to_string(),
            episode_id: "ep_summary_test".into(),
            event_type: EpisodeEventType::Action,
            description: "Set up project scaffolding".into(),
            agent_id: Some("agent_1".into()),
            timestamp: Utc::now(),
            metadata: serde_json::json!({}),
        },
        crate::episodic::models::EpisodeEvent {
            id: Uuid::now_v7().to_string(),
            episode_id: "ep_summary_test".into(),
            event_type: EpisodeEventType::Milestone,
            description: "All tests passing".into(),
            agent_id: None,
            timestamp: Utc::now(),
            metadata: serde_json::json!({}),
        },
    ];

    for ev in &events {
        episodic.store_event(ev).unwrap();
    }

    let summary = episodic.summarize_episode("ep_summary_test").unwrap();
    assert!(!summary.summary_text.is_empty());
    assert!(!summary.key_insights.is_empty());

    // Verify it was persisted
    let fetched = episodic.get_summary("ep_summary_test").unwrap();
    assert!(fetched.is_some());
}

/// Semantic extraction entity and relationship pipeline.
#[test]
fn semantic_extraction_pipeline() {
    let (store, _dir) = test_store();
    let semantic = SemanticMemoryEngine::new(store);

    let text = "The authentication service uses JWT tokens and requires OAuth integration. \
                It connects to Redis for caching and depends on PostgreSQL for persistence.";

    let extraction = semantic.extract_concepts(text, None).unwrap();

    // Should find several entities
    let entity_names: Vec<&str> = extraction
        .entities
        .iter()
        .map(|e| e.name.as_str())
        .collect();
    assert!(entity_names.len() >= 3);

    // Should find relationships
    assert!(!extraction.relationships.is_empty());

    // Should generate facts
    assert!(!extraction.facts.is_empty());

    // Store and verify persistence
    let ids = semantic.store_extraction(&extraction, None).unwrap();
    assert_eq!(semantic.count().unwrap(), ids.len() as u64);
}

/// Evolution: reinforce, decay, contradiction detection lifecycle.
#[test]
fn evolution_lifecycle() {
    let (store, _dir) = test_store();
    let semantic = SemanticMemoryEngine::new(store.clone());
    let evolution = KnowledgeEvolutionEngine::with_defaults(store);

    // Setup memories
    let extraction = semantic
        .extract_concepts("Authentication uses JWT tokens", None)
        .unwrap();
    let ids = semantic.store_extraction(&extraction, None).unwrap();

    // Reinforce
    let entry = evolution
        .reinforce(&ids[0], 0.05, "Confirmed", None)
        .unwrap();
    assert!(entry.new_confidence > entry.old_confidence);

    // Get history
    let history = evolution.get_history(&ids[0]).unwrap();
    assert_eq!(history.len(), 1);

    // Apply decay
    let decay_result = evolution.apply_decay(&ids[0], 50.0).unwrap();
    // May or may not produce a change depending on confidence and rate
    if let Some(entry) = decay_result {
        assert!(entry.new_confidence <= entry.old_confidence);
    }
}

/// Retrieval logs are persisted for auditing.
#[test]
fn retrieval_logging() {
    let (store, _dir) = test_store();
    let retrieval = RetrievalEngine::new(store);

    // Run a few queries
    for i in 0..3 {
        let req = RetrievalRequest {
            query_text: format!("query {}", i),
            ..Default::default()
        };
        retrieval.retrieve(&req).unwrap();
    }

    let logs = retrieval.recent_logs(10).unwrap();
    assert_eq!(logs.len(), 3);
    assert_eq!(retrieval.count_logs().unwrap(), 3);
}

/// Episode ranking orders by composite score.
#[test]
fn episode_ranking() {
    let (store, _dir) = test_store();
    let episodic = EpisodicMemoryEngine::new(store);

    let mut ep_high = make_test_episode("ep_rank_high", EpisodeOutcome::Success);
    ep_high.score = 0.95;
    ep_high.lessons_learned = vec!["L1".into(), "L2".into(), "L3".into()];

    let mut ep_low = make_test_episode("ep_rank_low", EpisodeOutcome::Failure);
    ep_low.score = 0.1;
    ep_low.lessons_learned = vec![];

    let ranked = episodic.rank_episodes(&[ep_high, ep_low]);
    assert_eq!(ranked.len(), 2);
    assert!(ranked[0].relevance_score >= ranked[1].relevance_score);
    assert_eq!(ranked[0].episode.id, "ep_rank_high");
}
