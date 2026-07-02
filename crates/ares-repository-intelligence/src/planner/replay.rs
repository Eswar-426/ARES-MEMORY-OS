use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::context::CachePolicy;
use crate::planner::intent::Intent;

/// Rich planner replay record. Stored to disk at
/// `.ares/planner/replay/YYYY-MM-DD/execution_{id}.json`
/// rather than embedded in every response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerReplay {
    pub execution_id: String,
    pub intent: String,
    pub repository: String,
    pub planner_version: String,
    pub context_hash: String,
    pub engines: Vec<EngineReplayEntry>,
    pub execution_graph: ExecutionGraphReplay,
    pub timings: HashMap<String, u64>,
    pub cache: CacheReplay,
    pub artifacts: Vec<String>,
    pub result: ReplayResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineReplayEntry {
    pub engine_id: String,
    pub version: String,
    pub capability: String,
    pub duration_ms: u64,
    pub success: bool,
    pub evidence_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionGraphReplay {
    pub node_count: usize,
    pub edge_count: usize,
    pub parallel_groups: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheReplay {
    pub plan_cache_hit: bool,
    pub engine_cache_hits: HashMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplayResult {
    Success,
    PartialFailure { failed_engines: Vec<String> },
    Failure { error: String },
}

/// Query Planner Cache — caches execution plans by
/// (intent, repository_id, policy, planner_version).
/// Avoids re-running builder, expander, optimizer, resolver
/// for identical questions.
pub struct PlannerCache {
    cache: HashMap<String, CachedPlan>,
}

#[derive(Debug, Clone)]
pub struct CachedPlan {
    pub plan_id: String,
    pub capabilities: Vec<crate::core::capabilities::Capability>,
    pub created_at: u64,
}

impl Default for PlannerCache {
    fn default() -> Self {
        Self::new()
    }
}

impl PlannerCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Generate a cache key from intent + repository_id + cache_policy + planner_version
    pub fn cache_key(
        intent: &Intent,
        repository_id: &str,
        cache_policy: &CachePolicy,
        planner_version: &str,
    ) -> String {
        format!(
            "{:?}:{}:{:?}:{}",
            intent, repository_id, cache_policy, planner_version
        )
    }

    pub fn get(&self, key: &str) -> Option<&CachedPlan> {
        self.cache.get(key)
    }

    pub fn insert(&mut self, key: String, plan: CachedPlan) {
        self.cache.insert(key, plan);
    }

    pub fn invalidate(&mut self) {
        self.cache.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}
