use crate::engines::overview::models::IntegrityOverview;
use ares_store::Store;

pub async fn collect(store: &Store) -> IntegrityOverview {
    let stats = store
        .overview_integrity_stats()
        .unwrap_or(ares_store::overview::IntegrityStats {
            missing_sources: 0,
            missing_targets: 0,
            orphans: 0,
        });

    let cycles = 0; // Expensive to compute via SQL, placeholder for MVP

    IntegrityOverview {
        foreign_keys_passed: stats.missing_sources == 0 && stats.missing_targets == 0,
        missing_targets: stats.missing_targets,
        missing_sources: stats.missing_sources,
        orphans: stats.orphans,
        cycles,
    }
}
