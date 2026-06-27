use crate::engines::overview::models::CoverageOverview;
use ares_store::Store;

pub async fn collect(store: &Store) -> CoverageOverview {
    let stats = store.overview_coverage_stats().unwrap_or_else(|_| ares_store::overview::CoverageStats {
        adrs: 0,
        requirements: 0,
        architecture_docs: 0,
        decisions: 0,
    });
        
    CoverageOverview {
        git_history_enabled: true,
        architecture_docs: stats.architecture_docs,
        requirements: stats.requirements,
        ownership_enabled: true,
        explicit_docs: 0,
        adrs: stats.adrs,
        decisions: stats.decisions,
        policies: 0,
    }
}
