use crate::engines::overview::models::IntelligenceOverview;
use ares_store::Store;

pub async fn collect(_store: &Store) -> IntelligenceOverview {
    IntelligenceOverview {
        why_exists_status: "READY".to_string(),
        graph_status: "READY".to_string(),
        git_memory_status: "READY".to_string(),
        ownership_status: "NOT AVAILABLE".to_string(),
        requirements_status: "NOT AVAILABLE".to_string(),
        governance_status: "NOT AVAILABLE".to_string(),
        impact_status: "READY".to_string(),
        traceability_status: "READY".to_string(),
        simulation_status: "READY".to_string(),
        drift_status: "READY".to_string(),
        last_query: None,
        last_query_time: None,
    }
}
