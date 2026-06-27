use super::models::{RepositoryDashboardResponse, CacheStats};
use ares_store::Store;
use std::sync::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use chrono::{DateTime, Utc};

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

pub struct RepositoryOverviewEngine;

impl RepositoryOverviewEngine {
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
                let hit_rate = if hits + misses > 0.0 { (hits / (hits + misses) * 100.0) } else { 0.0 };

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
            let hit_rate = if hits + misses > 0.0 { (hits / (hits + misses) * 100.0) } else { 0.0 };
            
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
            reasons: contributions.into_iter().map(|c| format!("{} {}", c.severity, c.title)).collect(),
        };

        // Update cache
        {
            let mut cache = DASHBOARD_CACHE.write().unwrap();
            cache.insert(project_path, CacheEntry {
                response: resp.clone(),
                generated_at: now,
                is_refreshing: false,
            });
        }

        resp
    }
}
