//! Graph intelligence — bridges the knowledge graph with the memory intelligence layer.
//!
//! Provides enrichment of the knowledge graph with episodic and semantic memories,
//! and query capabilities that combine graph traversal with memory retrieval.

use ares_core::AresError;
use ares_memory_intelligence::episodic::engine::EpisodicMemoryEngine;
use ares_memory_intelligence::episodic::models::EpisodeQuery;
use ares_memory_intelligence::semantic::engine::SemanticMemoryEngine;
use ares_memory_intelligence::semantic::models::SemanticQuery;
use ares_store::db::Store;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// A node in the memory-enriched knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryGraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String, // "entity", "episode", "concept", "principle"
    pub confidence: f64,
    pub metadata: serde_json::Value,
}

/// An edge in the memory-enriched knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryGraphEdge {
    pub source: String,
    pub target: String,
    pub relationship: String, // LEARNED_FROM, RESULTED_IN, RELATED_TO, etc.
    pub confidence: f64,
}

/// A subgraph extracted from memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySubgraph {
    pub nodes: Vec<MemoryGraphNode>,
    pub edges: Vec<MemoryGraphEdge>,
}

/// Service that enriches the knowledge graph with memory data.
pub struct MemoryGraphService {
    episodic: EpisodicMemoryEngine,
    semantic: SemanticMemoryEngine,
}

impl MemoryGraphService {
    pub fn new(store: Store) -> Self {
        Self {
            episodic: EpisodicMemoryEngine::new(store.clone()),
            semantic: SemanticMemoryEngine::new(store),
        }
    }

    /// Build a subgraph from an episode and its extracted semantics.
    pub fn build_episode_subgraph(&self, episode_id: &str) -> Result<MemorySubgraph, AresError> {
        let episode = self
            .episodic
            .get_episode(episode_id)?
            .ok_or_else(|| AresError::not_found("episode", episode_id))?;

        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Episode node
        nodes.push(MemoryGraphNode {
            id: episode.id.clone(),
            label: episode.title.clone(),
            node_type: "episode".into(),
            confidence: episode.score,
            metadata: serde_json::json!({
                "outcome": episode.outcome.as_str(),
                "agents": episode.agents_involved,
            }),
        });

        // Add agent nodes and INVOLVED_IN edges
        for agent in &episode.agents_involved {
            let agent_node_id = format!("agent_{}", agent);
            nodes.push(MemoryGraphNode {
                id: agent_node_id.clone(),
                label: agent.clone(),
                node_type: "entity".into(),
                confidence: 1.0,
                metadata: serde_json::json!({"type": "agent"}),
            });
            edges.push(MemoryGraphEdge {
                source: agent_node_id,
                target: episode.id.clone(),
                relationship: "INVOLVED_IN".into(),
                confidence: 1.0,
            });
        }

        // Query semantic memories linked to this episode
        let semantic_query = SemanticQuery {
            limit: Some(20),
            ..Default::default()
        };
        let memories = self.semantic.query(&semantic_query)?;

        for mem in &memories {
            if mem.source_episode_id.as_deref() == Some(episode_id) {
                nodes.push(MemoryGraphNode {
                    id: mem.id.clone(),
                    label: mem.subject.clone(),
                    node_type: "concept".into(),
                    confidence: mem.confidence,
                    metadata: serde_json::json!({
                        "predicate": mem.predicate,
                        "object": mem.object,
                    }),
                });
                edges.push(MemoryGraphEdge {
                    source: mem.id.clone(),
                    target: episode.id.clone(),
                    relationship: "LEARNED_FROM".into(),
                    confidence: mem.confidence,
                });
            }
        }

        // Add lesson nodes
        for (i, lesson) in episode.lessons_learned.iter().enumerate() {
            let lesson_id = format!("lesson_{}_{}", episode.id, i);
            nodes.push(MemoryGraphNode {
                id: lesson_id.clone(),
                label: lesson.clone(),
                node_type: "concept".into(),
                confidence: episode.score,
                metadata: serde_json::json!({"type": "lesson"}),
            });
            edges.push(MemoryGraphEdge {
                source: lesson_id,
                target: episode.id.clone(),
                relationship: "RESULTED_IN".into(),
                confidence: episode.score,
            });
        }

        debug!(
            nodes = nodes.len(),
            edges = edges.len(),
            "Built episode subgraph"
        );

        Ok(MemorySubgraph { nodes, edges })
    }

    /// Build a topic-centered subgraph from semantic memories.
    pub fn build_topic_subgraph(&self, topic: &str) -> Result<MemorySubgraph, AresError> {
        let query = SemanticQuery {
            subject: Some(topic.into()),
            ..Default::default()
        };
        let memories = self.semantic.query(&query)?;

        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Topic center node
        let center_id = format!("topic_{}", topic.to_lowercase().replace(' ', "_"));
        nodes.push(MemoryGraphNode {
            id: center_id.clone(),
            label: topic.into(),
            node_type: "concept".into(),
            confidence: 1.0,
            metadata: serde_json::json!({}),
        });

        for mem in &memories {
            nodes.push(MemoryGraphNode {
                id: mem.id.clone(),
                label: format!("{} {} {}", mem.subject, mem.predicate, mem.object),
                node_type: "concept".into(),
                confidence: mem.confidence,
                metadata: serde_json::json!({
                    "memory_type": format!("{:?}", mem.memory_type),
                }),
            });
            edges.push(MemoryGraphEdge {
                source: mem.id.clone(),
                target: center_id.clone(),
                relationship: "RELATED_TO".into(),
                confidence: mem.confidence,
            });
        }

        Ok(MemorySubgraph { nodes, edges })
    }

    /// Get related episodes for a topic.
    pub fn get_related_episodes(
        &self,
        topic: &str,
        limit: u32,
    ) -> Result<Vec<MemoryGraphNode>, AresError> {
        let query = EpisodeQuery {
            search_text: Some(topic.into()),
            limit: Some(limit),
            ..Default::default()
        };
        let episodes = self.episodic.query_episodes(&query)?;

        Ok(episodes
            .iter()
            .map(|ep| MemoryGraphNode {
                id: ep.id.clone(),
                label: ep.title.clone(),
                node_type: "episode".into(),
                confidence: ep.score,
                metadata: serde_json::json!({
                    "outcome": ep.outcome.as_str(),
                    "tags": ep.tags,
                }),
            })
            .collect())
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

    fn make_service() -> (MemoryGraphService, Store, tempfile::TempDir) {
        let (store, dir) = test_store();
        let service = MemoryGraphService::new(store.clone());
        (service, store, dir)
    }

    #[test]
    fn build_episode_subgraph() {
        let (service, store, _dir) = make_service();
        let episodic = EpisodicMemoryEngine::new(store);
        let mut ep = make_test_episode("ep_graph", EpisodeOutcome::Success);
        ep.agents_involved = vec!["coder".into(), "reviewer".into()];
        ep.lessons_learned = vec!["Always run tests".into()];
        episodic.store_episode(&ep).unwrap();

        let subgraph = service.build_episode_subgraph("ep_graph").unwrap();
        // 1 episode + 2 agents + 1 lesson = 4 nodes
        assert_eq!(subgraph.nodes.len(), 4);
        // 2 INVOLVED_IN + 1 RESULTED_IN = 3 edges
        assert_eq!(subgraph.edges.len(), 3);
    }

    #[test]
    fn build_topic_subgraph_empty() {
        let (service, _, _dir) = make_service();
        let subgraph = service.build_topic_subgraph("nonexistent").unwrap();
        // At least the center node
        assert_eq!(subgraph.nodes.len(), 1);
    }

    #[test]
    fn build_topic_subgraph_with_semantics() {
        let (service, store, _dir) = make_service();
        let semantic = SemanticMemoryEngine::new(store);

        let extraction = semantic
            .extract_concepts("Authentication uses JWT for token management", None)
            .unwrap();
        semantic.store_extraction(&extraction, None).unwrap();

        let subgraph = service.build_topic_subgraph("Auth").unwrap();
        // Should have center node + any matching semantics
        assert!(!subgraph.nodes.is_empty());
    }

    #[test]
    fn get_related_episodes() {
        let (service, store, _dir) = make_service();
        let episodic = EpisodicMemoryEngine::new(store);
        let mut ep = make_test_episode("ep_related", EpisodeOutcome::Success);
        ep.title = "Deploy to production".into();
        episodic.store_episode(&ep).unwrap();

        let nodes = service.get_related_episodes("Deploy", 10).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].node_type, "episode");
    }

    #[test]
    fn build_episode_subgraph_nonexistent_fails() {
        let (service, _, _dir) = make_service();
        let result = service.build_episode_subgraph("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn memory_graph_node_serialization() {
        let node = MemoryGraphNode {
            id: "n_1".into(),
            label: "Auth".into(),
            node_type: "concept".into(),
            confidence: 0.9,
            metadata: serde_json::json!({}),
        };
        let json = serde_json::to_string(&node).unwrap();
        let back: MemoryGraphNode = serde_json::from_str(&json).unwrap();
        assert_eq!(back.label, "Auth");
    }

    #[test]
    fn memory_graph_edge_serialization() {
        let edge = MemoryGraphEdge {
            source: "a".into(),
            target: "b".into(),
            relationship: "LEARNED_FROM".into(),
            confidence: 0.85,
        };
        let json = serde_json::to_string(&edge).unwrap();
        let back: MemoryGraphEdge = serde_json::from_str(&json).unwrap();
        assert_eq!(back.relationship, "LEARNED_FROM");
    }

    #[test]
    fn memory_subgraph_serialization() {
        let sg = MemorySubgraph {
            nodes: vec![],
            edges: vec![],
        };
        let json = serde_json::to_string(&sg).unwrap();
        let back: MemorySubgraph = serde_json::from_str(&json).unwrap();
        assert!(back.nodes.is_empty());
    }
}
