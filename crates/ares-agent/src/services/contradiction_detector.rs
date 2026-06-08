use ares_core::id::new_id;
use ares_core::types::event::now_micros;
use ares_core::{AresError, ContradictionRecord, EdgeType, GraphEdge, NodeId, ProjectId};
use ares_store::repositories::graph::SqliteGraphRepository;
use ares_store::repositories::intelligence::SqliteIntelligenceRepository;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictionAnalysis {
    pub severity: f32,
    pub confidence: f32,
    pub affected_memories: Vec<String>,
    pub affected_decisions: Vec<String>,
    pub recommendations: Vec<String>,
}

pub struct ContradictionReasoner {
    // Repositories will be needed for clustering/root detection
}

impl ContradictionReasoner {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ContradictionReasoner {
    fn default() -> Self {
        Self::new()
    }
}

impl ContradictionReasoner {
    pub fn analyze(
        &self,
        _project_id: &ProjectId,
        _contradiction_ids: &[String],
    ) -> Result<ContradictionAnalysis, AresError> {
        // Implement contradiction reasoning
        // Active contradictions, Resolved contradictions, Severity scoring
        // Contradiction clustering, Root contradiction detection

        Ok(ContradictionAnalysis {
            severity: 0.8,
            confidence: 0.9,
            affected_memories: vec![],
            affected_decisions: vec![],
            recommendations: vec!["Review affected decisions".into()],
        })
    }
}

pub struct ContradictionDetector {
    graph_repo: Arc<SqliteGraphRepository>,
    intelligence_repo: Arc<SqliteIntelligenceRepository>,
}

impl ContradictionDetector {
    pub fn new(
        graph_repo: Arc<SqliteGraphRepository>,
        intelligence_repo: Arc<SqliteIntelligenceRepository>,
    ) -> Self {
        Self {
            graph_repo,
            intelligence_repo,
        }
    }

    /// In a real implementation, this would use embeddings or an LLM to compare nodes.
    /// For this deterministic testable implementation, we simulate contradiction detection
    /// based on conflicting properties or overlapping 'Impacts' without a clear 'Supersedes' resolution.
    pub fn detect_contradictions(
        &self,
        project_id: &ProjectId,
        nodes_to_check: &[NodeId],
    ) -> Result<Vec<ContradictionRecord>, AresError> {
        let mut results = Vec::new();

        // Very basic mock heuristic: if two nodes have same label and aren't connected by supersedes
        // This is purely placeholder logic for deterministic context assembly until LLM orchestrator is added.
        for (i, node_a_id) in nodes_to_check.iter().enumerate() {
            let node_a = match self.graph_repo.get_node(node_a_id)? {
                Some(n) => n,
                None => continue,
            };

            for node_b_id in nodes_to_check.iter().skip(i + 1) {
                let node_b = match self.graph_repo.get_node(node_b_id)? {
                    Some(n) => n,
                    None => continue,
                };

                // Example "dumb" logic: if labels are identical but properties differ
                if node_a.label == node_b.label && node_a.properties != node_b.properties {
                    // Check if they are already resolved via supersedes
                    let edges = self.graph_repo.get_edges_from(node_a_id)?;
                    let has_supersedes = edges
                        .iter()
                        .any(|e| e.edge_type == EdgeType::Supersedes && e.to_node_id == *node_b_id);

                    let edges_b = self.graph_repo.get_edges_from(node_b_id)?;
                    let is_superseded = edges_b
                        .iter()
                        .any(|e| e.edge_type == EdgeType::Supersedes && e.to_node_id == *node_a_id);

                    if !has_supersedes && !is_superseded {
                        // Conflict detected!
                        let reason = format!("Nodes {} and {} share label '{}' but have conflicting properties without resolution.", node_a_id.as_str(), node_b_id.as_str(), node_a.label);
                        let confidence = 0.8;

                        // 1. Record contradiction in intelligence repo
                        let record = self.intelligence_repo.record_contradiction(
                            project_id, node_a_id, node_b_id, &reason, confidence,
                        )?;

                        // 2. Add graph edge
                        let now = now_micros();
                        let edge = GraphEdge {
                            id: new_id(),
                            project_id: project_id.clone(),
                            from_node_id: node_a_id.clone(),
                            to_node_id: node_b_id.clone(),
                            edge_type: EdgeType::Contradicts,
                            weight: 1.0,
                            confidence,
                            source: "agent".to_string(),
                            valid_from: now,
                            valid_until: None,
                            created_at: now,
                        };

                        self.graph_repo.upsert_edge(edge)?;
                        results.push(record);
                    }
                }
            }
        }

        Ok(results)
    }
}
