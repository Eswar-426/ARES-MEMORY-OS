use super::models::{CacheStats, RepositoryDashboardResponse};
use ares_store::Store;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

struct CacheEntry {
    response: RepositoryDashboardResponse,
    generated_at: DateTime<Utc>,
    is_refreshing: bool,
}

lazy_static::lazy_static! {
    static ref DASHBOARD_CACHE: RwLock<HashMap<String, CacheEntry>> = RwLock::new(HashMap::new());
}

static CACHE_HITS: AtomicU64 = AtomicU64::new(0);
static CACHE_MISSES: AtomicU64 = AtomicU64::new(0);
const CACHE_TTL_SECONDS: i64 = 2; // 2 seconds TTL

pub struct RepositoryOverviewEngine {
    pub store: Store,
}

impl RepositoryOverviewEngine {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub async fn collect(store: &Store, project_path: &str) -> RepositoryDashboardResponse {
        let now = Utc::now();
        let (cached_resp, needs_refresh) = {
            let mut cache = DASHBOARD_CACHE.write().unwrap();
            if let Some(entry) = cache.get_mut(project_path) {
                let age_seconds = (now - entry.generated_at).num_milliseconds() as f32 / 1000.0;

                // Update hit rate
                CACHE_HITS.fetch_add(1, Ordering::Relaxed);
                let hits = CACHE_HITS.load(Ordering::Relaxed) as f32;
                let misses = CACHE_MISSES.load(Ordering::Relaxed) as f32;
                let hit_rate = if hits + misses > 0.0 {
                    hits / (hits + misses) * 100.0
                } else {
                    0.0
                };

                entry.response.cache_stats = Some(CacheStats {
                    hit_rate: format!("{:.1}%", hit_rate),
                    age: age_seconds,
                    ttl: CACHE_TTL_SECONDS as f32,
                });

                let is_expired = age_seconds >= CACHE_TTL_SECONDS as f32;

                let mut should_spawn_refresh = false;
                if is_expired && !entry.is_refreshing {
                    entry.is_refreshing = true;
                    should_spawn_refresh = true;
                }

                entry.response.refreshing = entry.is_refreshing;
                (Some(entry.response.clone()), should_spawn_refresh)
            } else {
                CACHE_MISSES.fetch_add(1, Ordering::Relaxed);
                (None, false)
            }
        };

        let store_clone = store.clone();
        let path_clone = project_path.to_string();

        if let Some(mut resp) = cached_resp {
            if needs_refresh {
                tokio::spawn(async move {
                    Self::refresh_cache(store_clone, path_clone).await;
                });
                resp.refreshing = true;
            }
            resp
        } else {
            // First time, block and collect
            let mut resp = Self::refresh_cache(store_clone, path_clone).await;

            // On first fetch, miss
            let hits = CACHE_HITS.load(Ordering::Relaxed) as f32;
            let misses = CACHE_MISSES.load(Ordering::Relaxed) as f32;
            let hit_rate = if hits + misses > 0.0 {
                hits / (hits + misses) * 100.0
            } else {
                0.0
            };

            resp.cache_stats = Some(CacheStats {
                hit_rate: format!("{:.1}%", hit_rate),
                age: 0.0,
                ttl: CACHE_TTL_SECONDS as f32,
            });
            resp.refreshing = false;
            resp
        }
    }

    async fn refresh_cache(store: Store, project_path: String) -> RepositoryDashboardResponse {
        let repository = super::collectors::repository::collect(&store, &project_path).await;
        let graph = super::collectors::graph::collect(&store).await;
        let integrity = super::collectors::integrity::collect(&store).await;
        let coverage = super::collectors::coverage::collect(&store).await;
        let intelligence = super::collectors::intelligence::collect(&store).await;
        let performance = super::collectors::performance::collect(&store).await;
        let activity = super::collectors::activity::collect(&store).await;
        let version = super::collectors::version::collect().await;

        let now = Utc::now();
        let mut resp = RepositoryDashboardResponse {
            schema_version: 1,
            generated_at: now.to_rfc3339(),
            refreshing: false,
            cache_stats: None,
            repository_id: project_path.clone(),
            repository,
            graph,
            integrity,
            coverage,
            intelligence,
            performance,
            health: super::models::HealthOverview {
                score: 100,
                status: "Unknown".to_string(),
                reasons: vec![],
            },
            activity,
            version,
        };

        // Compute Health using HealthRule
        let (score, contributions) = super::health::evaluate_health(&resp);

        let status = if score > 90 {
            "Healthy"
        } else if score > 70 {
            "Warning"
        } else {
            "Critical"
        }
        .to_string();

        resp.health = super::models::HealthOverview {
            score,
            status,
            reasons: contributions
                .into_iter()
                .map(|c| format!("{} {}", c.severity, c.title))
                .collect(),
        };

        // Update cache
        {
            let mut cache = DASHBOARD_CACHE.write().unwrap();
            cache.insert(
                project_path,
                CacheEntry {
                    response: resp.clone(),
                    generated_at: now,
                    is_refreshing: false,
                },
            );
        }

        resp
    }
}

use crate::core::capabilities::Capability;
use crate::core::context::RepositoryContext;
use crate::core::engine::{
    Artifact, EngineDescriptor, EngineExecutionResult, EngineId, EngineInput, RepositoryEngine,
};
use crate::core::errors::{EngineError, EngineResult};
use crate::core::evidence::{EvidenceBundle, RuntimeEvidence};
use crate::core::metadata::ExecutionMetadata;

#[async_trait::async_trait]
impl RepositoryEngine for RepositoryOverviewEngine {
    fn descriptor(&self) -> EngineDescriptor {
        EngineDescriptor {
            id: EngineId::Overview,
            version: "0.1.0".to_string(),
            capabilities: vec![Capability::Workspace],
            planner_api_version: 1,
        }
    }

    async fn execute(
        &self,
        context: &RepositoryContext,
        _input: EngineInput,
    ) -> EngineResult<EngineExecutionResult> {
        let start = std::time::Instant::now();
        let project_path = &context.repository.root_path;

        let response = Self::collect(&self.store, project_path).await;

        let mut diagnostics = std::collections::HashMap::new();
        diagnostics.insert(
            "health_score".to_string(),
            response.health.score.to_string(),
        );
        diagnostics.insert("health_status".to_string(), response.health.status.clone());

        let mut warnings = Vec::new();
        if response.health.score < 90 {
            warnings.extend(response.health.reasons.clone());
        }

        let json_content = serde_json::to_string(&response)
            .map_err(|e| EngineError::InternalError(e.to_string()))?;

        let artifacts = vec![Artifact {
            name: "overview_dashboard.json".to_string(),
            format: "JSON".to_string(),
            content: json_content,
        }];

        let mut evidence = EvidenceBundle::default();
        evidence.runtime = Some(RuntimeEvidence {
            confidence: (response.health.score as f32) / 100.0,
            statistics: std::collections::HashMap::new(),
            sources: vec!["RepositoryOverviewEngine".to_string()],
        });

        let metadata = ExecutionMetadata {
            engine: "RepositoryOverviewEngine".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
            cache_hit: response.cache_stats.map(|c| c.age > 0.0).unwrap_or(false),
            confidence: (response.health.score as f32) / 100.0,
            errors: vec![],
            warnings: warnings.clone(),
            retry_count: 0,
            sources_used: vec!["RepositoryOverviewEngine".to_string()],
        };

        Ok(EngineExecutionResult {
            descriptor: self.descriptor(),
            engine_id: EngineId::Overview,
            capability: Capability::Workspace,
            evidence,
            metadata,
            diagnostics,
            warnings,
            errors: vec![],
            artifacts,
        })
    }
}
