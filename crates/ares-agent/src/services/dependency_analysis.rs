use ares_core::{AresError, ImpactEntry, NodeId};
use ares_store::repositories::graph::SqliteGraphRepository;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    pub impacts: Vec<ImpactEntry>,
    pub depth_reached: u8,
    pub confidence: f32,
    pub cycle_detected: bool,
}

pub struct DependencyAnalyzer {
    graph_repo: Arc<SqliteGraphRepository>,
}

impl DependencyAnalyzer {
    pub fn new(graph_repo: Arc<SqliteGraphRepository>) -> Self {
        Self { graph_repo }
    }

    /// Analyze impacts caused by this node (what depends on this node).
    /// Uses recommended defaults if depth is not provided (Impact analysis: depth 3)
    pub fn impacts(
        &self,
        start_node: &NodeId,
        depth_limit: Option<u8>,
    ) -> Result<DependencyAnalysis, AresError> {
        let depth = depth_limit.unwrap_or(3);
        let impact_graph = self.graph_repo.traverse_impact(start_node, depth)?;

        // Calculate aggregate confidence based on paths
        let avg_confidence = if impact_graph.impacts.is_empty() {
            1.0
        } else {
            let sum: f32 = impact_graph.impacts.iter().map(|i| i.confidence).sum();
            sum / impact_graph.impacts.len() as f32
        };

        // For now, traverse_impact inherently avoids infinite cycles via depth cap and DISTINCT constraints,
        // but we can flag if we hit the cap.
        let max_dist = impact_graph
            .impacts
            .iter()
            .map(|i| i.distance)
            .max()
            .unwrap_or(0);
        let cycle_detected = max_dist == depth;

        Ok(DependencyAnalysis {
            impacts: impact_graph.impacts,
            depth_reached: max_dist,
            confidence: avg_confidence,
            cycle_detected,
        })
    }

    /// Analyze what this node is impacted by (what this node depends on).
    pub fn impacted_by(
        &self,
        _start_node: &NodeId,
        _depth_limit: Option<u8>,
    ) -> Result<DependencyAnalysis, AresError> {
        // In a full implementation, this would run a reverse traversal (incoming edges).
        // Since traverse_impact in graph repo currently goes forward, we mock the reverse for now.
        Ok(DependencyAnalysis {
            impacts: vec![],
            depth_reached: 0,
            confidence: 1.0,
            cycle_detected: false,
        })
    }

    pub fn transitive_dependencies(
        &self,
        start_node: &NodeId,
    ) -> Result<DependencyAnalysis, AresError> {
        // Deep analysis: depth 5
        self.impacts(start_node, Some(5))
    }

    pub fn critical_path(&self, start_node: &NodeId) -> Result<Vec<NodeId>, AresError> {
        let analysis = self.impacts(start_node, Some(5))?;
        // Just an example: sorted by distance and returning IDs
        let mut path = vec![start_node.clone()];
        let mut sorted = analysis.impacts;
        sorted.sort_by_key(|i| i.distance);
        for item in sorted {
            path.push(item.node.id);
        }
        Ok(path)
    }

    pub fn dependency_depth(&self, start_node: &NodeId) -> Result<u8, AresError> {
        let analysis = self.impacts(start_node, Some(5))?;
        Ok(analysis.depth_reached)
    }
}
