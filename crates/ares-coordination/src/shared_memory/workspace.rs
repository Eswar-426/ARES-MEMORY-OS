use ares_agent_runtime::models::{AgentId, MissionId, TeamId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::fact::{FactCategory, FactId, SharedFact};

/// Thread-safe shared workspace scoped to a team/mission.
pub struct SharedWorkspace {
    pub mission_id: Option<MissionId>,
    pub team_id: Option<TeamId>,
    facts: Arc<RwLock<HashMap<FactId, SharedFact>>>,
}

impl SharedWorkspace {
    pub fn new() -> Self {
        Self {
            mission_id: None,
            team_id: None,
            facts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn for_mission(mission_id: MissionId) -> Self {
        Self {
            mission_id: Some(mission_id),
            team_id: None,
            facts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn for_team(team_id: TeamId) -> Self {
        Self {
            mission_id: None,
            team_id: Some(team_id),
            facts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Publish a fact to the workspace.
    pub async fn publish_fact(&self, fact: SharedFact) -> FactId {
        let id = fact.id;
        self.facts.write().await.insert(id, fact);
        id
    }

    /// Query facts by category.
    pub async fn query_by_category(&self, category: &FactCategory) -> Vec<SharedFact> {
        self.facts
            .read()
            .await
            .values()
            .filter(|f| f.is_active() && f.category == *category)
            .cloned()
            .collect()
    }

    /// Query facts by key prefix.
    pub async fn query_by_key_prefix(&self, prefix: &str) -> Vec<SharedFact> {
        self.facts
            .read()
            .await
            .values()
            .filter(|f| f.is_active() && f.key.starts_with(prefix))
            .cloned()
            .collect()
    }

    /// Query all active facts by a specific author.
    pub async fn query_by_author(&self, author: &AgentId) -> Vec<SharedFact> {
        self.facts
            .read()
            .await
            .values()
            .filter(|f| f.is_active() && f.author == *author)
            .cloned()
            .collect()
    }

    /// Get a specific fact by ID.
    pub async fn get_fact(&self, id: &FactId) -> Option<SharedFact> {
        self.facts.read().await.get(id).cloned()
    }

    /// Retract a fact by ID.
    pub async fn retract_fact(&self, id: &FactId) -> Result<(), String> {
        let mut facts = self.facts.write().await;
        if let Some(fact) = facts.get_mut(id) {
            fact.retract();
            Ok(())
        } else {
            Err(format!("Fact {:?} not found", id))
        }
    }

    /// Update a fact's value.
    pub async fn update_fact(&self, id: &FactId, value: impl Into<String>) -> Result<(), String> {
        let mut facts = self.facts.write().await;
        if let Some(fact) = facts.get_mut(id) {
            fact.update_value(value);
            Ok(())
        } else {
            Err(format!("Fact {:?} not found", id))
        }
    }

    /// Get all active facts.
    pub async fn all_active_facts(&self) -> Vec<SharedFact> {
        self.facts
            .read()
            .await
            .values()
            .filter(|f| f.is_active())
            .cloned()
            .collect()
    }

    /// Get total fact count (including retracted).
    pub async fn fact_count(&self) -> usize {
        self.facts.read().await.len()
    }

    /// Get active fact count.
    pub async fn active_fact_count(&self) -> usize {
        self.facts
            .read()
            .await
            .values()
            .filter(|f| f.is_active())
            .count()
    }

    /// Take a snapshot of all active facts (for consensus/debate).
    pub async fn snapshot(&self) -> Vec<SharedFact> {
        self.all_active_facts().await
    }

    /// Clear all facts.
    pub async fn clear(&self) {
        self.facts.write().await.clear();
    }
}

impl Default for SharedWorkspace {
    fn default() -> Self {
        Self::new()
    }
}
