use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A fact learned by an agent during execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLearnedFact {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub key: String,
    pub value: String,
    pub confidence: f64,
    pub source: String,
    pub learned_at: i64,
}

impl AgentLearnedFact {
    pub fn new(
        agent_id: Uuid,
        key: impl Into<String>,
        value: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            agent_id,
            key: key.into(),
            value: value.into(),
            confidence: 1.0,
            source: source.into(),
            learned_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }
}

/// A higher-level insight derived from multiple facts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDiscovery {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub title: String,
    pub description: String,
    pub supporting_facts: Vec<Uuid>,
    pub confidence: f64,
    pub discovered_at: i64,
}

impl AgentDiscovery {
    pub fn new(
        agent_id: Uuid,
        title: impl Into<String>,
        description: impl Into<String>,
        supporting_facts: Vec<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            agent_id,
            title: title.into(),
            description: description.into(),
            supporting_facts,
            confidence: 0.8,
            discovered_at: chrono::Utc::now().timestamp(),
        }
    }
}

/// A recommendation for future tasks based on experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecommendation {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub task_type: String,
    pub recommendation: String,
    pub confidence: f64,
    pub created_at: i64,
}

impl AgentRecommendation {
    pub fn new(
        agent_id: Uuid,
        task_type: impl Into<String>,
        recommendation: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            agent_id,
            task_type: task_type.into(),
            recommendation: recommendation.into(),
            confidence: 0.7,
            created_at: chrono::Utc::now().timestamp(),
        }
    }
}

/// Synchronizes agent knowledge with the global knowledge store.
pub struct KnowledgeSynchronizer {
    facts: HashMap<Uuid, AgentLearnedFact>,
    discoveries: HashMap<Uuid, AgentDiscovery>,
    recommendations: HashMap<Uuid, AgentRecommendation>,
}

impl KnowledgeSynchronizer {
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
            discoveries: HashMap::new(),
            recommendations: HashMap::new(),
        }
    }

    /// Sync agent facts into global knowledge.
    pub fn sync_agent_facts(&mut self, facts: Vec<AgentLearnedFact>) -> usize {
        let count = facts.len();
        for fact in facts {
            self.facts.insert(fact.id, fact);
        }
        count
    }

    /// Propagate a discovery to relevant teams.
    pub fn propagate_discovery(&mut self, discovery: AgentDiscovery) -> Uuid {
        let id = discovery.id;
        self.discoveries.insert(id, discovery);
        id
    }

    /// Add a recommendation.
    pub fn add_recommendation(&mut self, recommendation: AgentRecommendation) -> Uuid {
        let id = recommendation.id;
        self.recommendations.insert(id, recommendation);
        id
    }

    /// Get recommendations for a task type.
    pub fn get_recommendations(&self, task_type: &str) -> Vec<&AgentRecommendation> {
        self.recommendations
            .values()
            .filter(|r| r.task_type == task_type)
            .collect()
    }

    /// Get all facts from a specific agent.
    pub fn get_agent_facts(&self, agent_id: &Uuid) -> Vec<&AgentLearnedFact> {
        self.facts
            .values()
            .filter(|f| f.agent_id == *agent_id)
            .collect()
    }

    /// Get total fact count.
    pub fn fact_count(&self) -> usize {
        self.facts.len()
    }

    /// Get total discovery count.
    pub fn discovery_count(&self) -> usize {
        self.discoveries.len()
    }

    /// Get total recommendation count.
    pub fn recommendation_count(&self) -> usize {
        self.recommendations.len()
    }
}

impl Default for KnowledgeSynchronizer {
    fn default() -> Self {
        Self::new()
    }
}
