//! Week 8 — Agent Registry: deterministic capability index, health scoring,
//! performance ranking, and agent lifecycle management.
//!
//! Uses BTreeMap/BTreeSet exclusively for deterministic ordering.
//! Ranking order: health_score → success_rate → latency (lower) → AgentId.

use ares_core::types::event::now_micros;
use ares_core::{AgentHealth, AgentId, AgentInfo, AresError};
use std::collections::{BTreeMap, BTreeSet};

// ─────────────────────────────────────────────────────────────────
// Agent capability index (deterministic, in-memory cache)
// ─────────────────────────────────────────────────────────────────

/// Deterministic in-memory index of agent capabilities.
/// All collections use BTree variants for ordering guarantees.
pub struct AgentCapabilityIndex {
    /// Capability name → set of agent IDs that provide it.
    capability_to_agents: BTreeMap<String, BTreeSet<AgentId>>,
    /// Agent ID → cached health score (0.0–1.0).
    agent_health_cache: BTreeMap<AgentId, f64>,
    /// Agent ID → cached performance data (success_rate, avg_latency_ms).
    performance_cache: BTreeMap<AgentId, (f64, f64)>,
    /// Agent ID → full agent info.
    agent_info_cache: BTreeMap<AgentId, AgentInfo>,
    /// Last cache refresh timestamp (Unix microseconds).
    last_refresh: i64,
}

impl AgentCapabilityIndex {
    /// Create an empty index.
    pub fn new() -> Self {
        Self {
            capability_to_agents: BTreeMap::new(),
            agent_health_cache: BTreeMap::new(),
            performance_cache: BTreeMap::new(),
            agent_info_cache: BTreeMap::new(),
            last_refresh: 0,
        }
    }

    /// Build the index from a list of agents.
    pub fn build_from(agents: &[AgentInfo]) -> Self {
        let mut index = Self::new();
        for agent in agents {
            index.insert(agent);
        }
        index.last_refresh = now_micros();
        index
    }

    /// Insert or update an agent in the index.
    pub fn insert(&mut self, agent: &AgentInfo) {
        for cap in &agent.capabilities {
            self.capability_to_agents
                .entry(cap.clone())
                .or_default()
                .insert(agent.id.clone());
        }
        self.agent_health_cache
            .insert(agent.id.clone(), agent.health.health_score);
        self.performance_cache.insert(
            agent.id.clone(),
            (
                agent.performance.success_rate,
                agent.performance.avg_latency_ms,
            ),
        );
        self.agent_info_cache
            .insert(agent.id.clone(), agent.clone());
    }

    /// Remove an agent from the index.
    pub fn remove(&mut self, agent_id: &AgentId) {
        // Remove from capability map
        let mut empty_caps = vec![];
        for (cap, agents) in self.capability_to_agents.iter_mut() {
            agents.remove(agent_id);
            if agents.is_empty() {
                empty_caps.push(cap.clone());
            }
        }
        for cap in empty_caps {
            self.capability_to_agents.remove(&cap);
        }
        self.agent_health_cache.remove(agent_id);
        self.performance_cache.remove(agent_id);
        self.agent_info_cache.remove(agent_id);
    }

    /// Find agents that have a specific capability.
    pub fn agents_for_capability(&self, capability: &str) -> Vec<AgentId> {
        self.capability_to_agents
            .get(capability)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Find the best agent for a capability using deterministic ranking.
    ///
    /// Ranking order:
    /// 1. health_score (higher is better)
    /// 2. success_rate (higher is better)
    /// 3. avg_latency_ms (lower is better)
    /// 4. AgentId (lexicographic — deterministic tie-break)
    pub fn find_best_agent(&self, capability: &str) -> Option<AgentId> {
        let candidates = self.agents_for_capability(capability);
        if candidates.is_empty() {
            return None;
        }

        let mut ranked: Vec<(AgentId, f64, f64, f64)> = candidates
            .into_iter()
            .filter_map(|id| {
                let health = self.agent_health_cache.get(&id).copied().unwrap_or(0.0);
                let (success_rate, latency) = self
                    .performance_cache
                    .get(&id)
                    .copied()
                    .unwrap_or((0.0, f64::MAX));
                // Only consider available agents
                let info = self.agent_info_cache.get(&id)?;
                if !info.health.is_available {
                    return None;
                }
                Some((id, health, success_rate, latency))
            })
            .collect();

        // Sort: higher health first, then higher success, then lower latency, then AgentId
        ranked.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal))
                .then_with(|| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal))
                .then_with(|| a.0.as_str().cmp(b.0.as_str()))
        });

        ranked.first().map(|(id, _, _, _)| id.clone())
    }

    /// Get agent count.
    pub fn agent_count(&self) -> usize {
        self.agent_info_cache.len()
    }

    /// Get all capability names.
    pub fn capabilities(&self) -> Vec<String> {
        self.capability_to_agents.keys().cloned().collect()
    }

    /// Get last refresh timestamp.
    pub fn last_refresh(&self) -> i64 {
        self.last_refresh
    }
}

impl Default for AgentCapabilityIndex {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────
// Agent Registry (persistent + cached)
// ─────────────────────────────────────────────────────────────────

use ares_store::repositories::traits::WorkflowRepository;
use std::sync::{Arc, RwLock};

pub struct AgentRegistry {
    repo: Arc<dyn WorkflowRepository + Send + Sync>,
    index: RwLock<AgentCapabilityIndex>,
}

impl AgentRegistry {
    pub fn new(repo: Arc<dyn WorkflowRepository + Send + Sync>) -> Result<Self, AresError> {
        let agents = repo.list_agents()?;
        let index = RwLock::new(AgentCapabilityIndex::build_from(&agents));
        Ok(Self { repo, index })
    }

    /// Register a new agent (persists + updates index).
    pub fn register(&self, agent: AgentInfo) -> Result<(), AresError> {
        let caps_json = serde_json::to_string(&agent.capabilities)?;
        let health_json = serde_json::to_string(&agent.health)?;
        let perf_json = serde_json::to_string(&agent.performance)?;

        // DB Call - NO LOCK HELD
        self.repo.register_agent(
            agent.id.as_str(),
            &agent.name,
            &caps_json,
            &health_json,
            &perf_json,
        )?;

        // Update cache - LOCK HELD BRIEFLY
        self.index.write().unwrap().insert(&agent);
        Ok(())
    }

    /// Find the best agent for a capability.
    pub fn find_best_agent(&self, capability: &str) -> Option<AgentId> {
        self.index.read().unwrap().find_best_agent(capability)
    }

    /// Update agent health.
    pub fn health_check(&self, agent_id: &AgentId, health: AgentHealth) -> Result<(), AresError> {
        let health_json = serde_json::to_string(&health)?;

        // DB Call - NO LOCK HELD
        self.repo
            .update_agent_health(agent_id.as_str(), &health_json)?;

        // Update cache - LOCK HELD BRIEFLY
        let mut index = self.index.write().unwrap();
        index
            .agent_health_cache
            .insert(agent_id.clone(), health.health_score);
        if let Some(info) = index.agent_info_cache.get_mut(agent_id) {
            info.health = health;
        }

        Ok(())
    }

    /// Rank agents by performance for a given capability.
    pub fn performance_rank(&self, capability: &str) -> Vec<AgentId> {
        let index = self.index.read().unwrap();
        let candidates = index.agents_for_capability(capability);

        let mut ranked: Vec<(AgentId, f64, f64, f64)> = candidates
            .into_iter()
            .map(|id| {
                let health = index.agent_health_cache.get(&id).copied().unwrap_or(0.0);
                let (success_rate, latency) = index
                    .performance_cache
                    .get(&id)
                    .copied()
                    .unwrap_or((0.0, f64::MAX));
                (id, health, success_rate, latency)
            })
            .collect();
        drop(index);

        ranked.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal))
                .then_with(|| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal))
                .then_with(|| a.0.as_str().cmp(b.0.as_str()))
        });

        ranked.into_iter().map(|(id, _, _, _)| id).collect()
    }

    /// Refresh the index from the database.
    pub fn refresh(&mut self) -> Result<(), AresError> {
        let agents = self.repo.list_agents()?;
        *self.index.write().unwrap() = AgentCapabilityIndex::build_from(&agents);
        Ok(())
    }

    /// Get agent count.
    pub fn agent_count(&self) -> usize {
        self.index.read().unwrap().agent_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::test_helpers::test_store;
    use ares_core::AgentPerformance;
    use ares_core::{AgentHealth, AgentId};
    use ares_store::SqliteWorkflowRepository;
    use std::sync::Arc;

    fn make_agent(
        name: &str,
        caps: &[&str],
        health_score: f64,
        success_rate: f64,
        latency: f64,
    ) -> AgentInfo {
        AgentInfo {
            id: AgentId::from(name),
            name: name.into(),
            capabilities: caps.iter().map(|s| s.to_string()).collect(),
            health: AgentHealth {
                health_score,
                is_available: true,
                last_check: now_micros(),
                consecutive_failures: 0,
            },
            performance: AgentPerformance {
                total_tasks: 100,
                successful_tasks: (success_rate * 100.0) as u64,
                failed_tasks: ((1.0 - success_rate) * 100.0) as u64,
                avg_latency_ms: latency,
                success_rate,
            },
            registered_at: now_micros(),
        }
    }

    #[test]
    fn capability_index_basics() {
        let agents = vec![
            make_agent("agent-a", &["build", "test"], 1.0, 0.95, 50.0),
            make_agent("agent-b", &["build", "deploy"], 0.8, 0.90, 100.0),
            make_agent("agent-c", &["test"], 0.9, 0.99, 30.0),
        ];

        let index = AgentCapabilityIndex::build_from(&agents);

        assert_eq!(index.agent_count(), 3);
        assert_eq!(index.agents_for_capability("build").len(), 2);
        assert_eq!(index.agents_for_capability("test").len(), 2);
        assert_eq!(index.agents_for_capability("deploy").len(), 1);
        assert_eq!(index.agents_for_capability("unknown").len(), 0);
    }

    #[test]
    fn find_best_agent_deterministic() {
        let agents = vec![
            make_agent("agent-a", &["build"], 1.0, 0.95, 50.0),
            make_agent("agent-b", &["build"], 0.8, 0.90, 100.0),
        ];

        let index = AgentCapabilityIndex::build_from(&agents);

        // agent-a has higher health (1.0 vs 0.8), should win
        let best = index.find_best_agent("build").unwrap();
        assert_eq!(best.as_str(), "agent-a");
    }

    #[test]
    fn find_best_agent_tie_break_by_success_rate() {
        let agents = vec![
            make_agent("agent-a", &["build"], 1.0, 0.90, 50.0),
            make_agent("agent-b", &["build"], 1.0, 0.95, 50.0),
        ];

        let index = AgentCapabilityIndex::build_from(&agents);

        // Same health, agent-b has higher success rate
        let best = index.find_best_agent("build").unwrap();
        assert_eq!(best.as_str(), "agent-b");
    }

    #[test]
    fn find_best_agent_tie_break_by_latency() {
        let agents = vec![
            make_agent("agent-a", &["build"], 1.0, 0.95, 100.0),
            make_agent("agent-b", &["build"], 1.0, 0.95, 50.0),
        ];

        let index = AgentCapabilityIndex::build_from(&agents);

        // Same health and success, agent-b has lower latency
        let best = index.find_best_agent("build").unwrap();
        assert_eq!(best.as_str(), "agent-b");
    }

    #[test]
    fn find_best_agent_tie_break_by_id() {
        let agents = vec![
            make_agent("agent-b", &["build"], 1.0, 0.95, 50.0),
            make_agent("agent-a", &["build"], 1.0, 0.95, 50.0),
        ];

        let index = AgentCapabilityIndex::build_from(&agents);

        // Everything equal — lexicographic AgentId wins
        let best = index.find_best_agent("build").unwrap();
        assert_eq!(best.as_str(), "agent-a");
    }

    #[test]
    fn unavailable_agents_excluded() {
        let mut agent = make_agent("agent-a", &["build"], 1.0, 0.95, 50.0);
        agent.health.is_available = false;

        let index = AgentCapabilityIndex::build_from(&[agent]);
        assert!(index.find_best_agent("build").is_none());
    }

    #[test]
    fn registry_with_persistence() {
        let (store, _dir) = test_store();
        let mut registry =
            AgentRegistry::new(Arc::new(SqliteWorkflowRepository::new(store))).unwrap();

        let agent = make_agent("agent-x", &["compile", "lint"], 0.95, 0.98, 45.0);
        registry.register(agent).unwrap();

        assert_eq!(registry.agent_count(), 1);
        assert_eq!(
            registry.find_best_agent("compile").unwrap().as_str(),
            "agent-x"
        );

        // Refresh from DB
        registry.refresh().unwrap();
        assert_eq!(registry.agent_count(), 1);
    }

    #[test]
    fn performance_rank_order() {
        let (store, _dir) = test_store();
        let registry = AgentRegistry::new(Arc::new(SqliteWorkflowRepository::new(store))).unwrap();

        registry
            .register(make_agent("slow", &["build"], 1.0, 0.90, 200.0))
            .unwrap();
        registry
            .register(make_agent("fast", &["build"], 1.0, 0.90, 20.0))
            .unwrap();
        registry
            .register(make_agent("best", &["build"], 1.0, 0.99, 10.0))
            .unwrap();

        let ranked = registry.performance_rank("build");
        // best: highest success_rate (0.99) wins first
        assert_eq!(ranked[0].as_str(), "best");
        // fast vs slow: same success_rate, lower latency wins
        assert_eq!(ranked[1].as_str(), "fast");
        assert_eq!(ranked[2].as_str(), "slow");
    }

    #[test]
    fn remove_agent_from_index() {
        let agents = vec![
            make_agent("agent-a", &["build"], 1.0, 0.95, 50.0),
            make_agent("agent-b", &["build"], 0.8, 0.90, 100.0),
        ];

        let mut index = AgentCapabilityIndex::build_from(&agents);
        assert_eq!(index.agent_count(), 2);

        index.remove(&AgentId::from("agent-a"));
        assert_eq!(index.agent_count(), 1);
        assert_eq!(index.find_best_agent("build").unwrap().as_str(), "agent-b");
    }
}
