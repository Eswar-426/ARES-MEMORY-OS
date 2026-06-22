use serde::{Deserialize, Serialize};
use ares_core::AresError;
use ares_store::{Store, SqliteGraphRepository};
use crate::classifier::{ArtifactClassifier, ArtifactCategory, MemoryEligibility};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MemoryDriftMetrics {
    pub artifacts_changed: u64,
    pub artifacts_changed_without_memory_updates: u64,
    pub memory_drift_percentage: f64,
}

pub struct MemoryDriftEngine;

impl MemoryDriftEngine {
    pub fn calculate(store: &Store, project_id: &ares_core::ProjectId) -> Result<MemoryDriftMetrics, AresError> {
        let repo = SqliteGraphRepository::new(store.clone());
        let nodes = repo.get_all_nodes(project_id)?;
        let edges = repo.get_all_edges(project_id)?;

        let mut artifacts_changed = 0;
        let mut artifacts_changed_without_memory_updates = 0;

        for node in &nodes {
            let classification = ArtifactClassifier::classify(Some(&node.node_type), node.file_path.as_deref());
            
            // Only care about eligible implementation nodes (e.g. Code)
            if classification.eligibility == MemoryEligibility::Required && classification.category == ArtifactCategory::Code {
                // If it hasn't been updated, skip
                if node.updated_at == node.created_at || node.updated_at == 0 {
                    continue;
                }
                
                artifacts_changed += 1;

                // Find upstream reasoning nodes
                let upstream_reasoning: Vec<_> = edges.iter()
                    .filter(|e| e.to_node_id == node.id && (e.edge_type == ares_core::EdgeType::Drives || e.edge_type == ares_core::EdgeType::Satisfies || e.edge_type == ares_core::EdgeType::Implements))
                    .filter_map(|e| nodes.iter().find(|n| n.id == e.from_node_id))
                    .collect();
                
                if upstream_reasoning.is_empty() {
                    // Changed but has no memory at all
                    artifacts_changed_without_memory_updates += 1;
                } else {
                    // If the code was updated much later than the reasoning (e.g., > 1 week difference), it's drifting.
                    // For a strict metric: did the reasoning update alongside the code?
                    // We check if the most recently updated reasoning node is older than the code update by some threshold.
                    // Let's use 30 days (in milliseconds) as a threshold for "drift".
                    let thirty_days_ms = 30 * 24 * 60 * 60 * 1000;
                    
                    let max_reasoning_update = upstream_reasoning.iter().map(|n| n.updated_at).max().unwrap_or(0);
                    
                    if node.updated_at > max_reasoning_update && (node.updated_at - max_reasoning_update) > thirty_days_ms {
                        artifacts_changed_without_memory_updates += 1;
                    }
                }
            }
        }

        let percentage = if artifacts_changed > 0 {
            (artifacts_changed_without_memory_updates as f64 / artifacts_changed as f64) * 100.0
        } else {
            0.0
        };

        Ok(MemoryDriftMetrics {
            artifacts_changed,
            artifacts_changed_without_memory_updates,
            memory_drift_percentage: percentage,
        })
    }
}
