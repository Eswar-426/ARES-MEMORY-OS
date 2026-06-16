use crate::graph::ReasoningGraph;
use crate::models::ImpactReport;
use ares_core::{EdgeType, NodeId};
use std::collections::HashSet;

pub struct ImpactAnalyzer;

impl ImpactAnalyzer {
    pub fn analyze(graph: &ReasoningGraph, target: &NodeId) -> ImpactReport {
        let mut direct_dependents = HashSet::new();
        let mut indirect_dependents = HashSet::new();
        let mut affected_files = HashSet::new();

        let mut queue = vec![target.clone()];
        let mut visited = HashSet::new();
        visited.insert(target.clone());

        if let Some(node) = graph.nodes.get(target) {
            if let Some(file_path) = &node.file_path {
                affected_files.insert(file_path.clone());
            }
        }

        // Helper to check if edge implies a dependency
        let is_dependency = |edge_type: &EdgeType| {
            matches!(
                edge_type,
                EdgeType::Imports
                    | EdgeType::DependsOn
                    | EdgeType::Calls
                    | EdgeType::Implements
                    | EdgeType::Uses
                    | EdgeType::Extends
            )
        };

        // direct dependents
        if let Some(incoming) = graph.incoming.get(target) {
            for edge in incoming {
                if !is_dependency(&edge.edge_type) {
                    continue;
                }
                direct_dependents.insert(edge.from_node_id.clone());
                queue.push(edge.from_node_id.clone());
                visited.insert(edge.from_node_id.clone());

                if let Some(node) = graph.nodes.get(&edge.from_node_id) {
                    if let Some(file_path) = &node.file_path {
                        affected_files.insert(file_path.clone());
                    }
                }
            }
        }

        let direct_count = direct_dependents.len();

        // indirect dependents
        let mut current_idx = 0;
        while current_idx < queue.len() {
            // skipping the first one since it's the target
            let current = queue[current_idx].clone();
            current_idx += 1;

            if let Some(incoming) = graph.incoming.get(&current) {
                for edge in incoming {
                    if !is_dependency(&edge.edge_type) {
                        continue;
                    }
                    if !visited.contains(&edge.from_node_id) {
                        visited.insert(edge.from_node_id.clone());
                        queue.push(edge.from_node_id.clone());
                        indirect_dependents.insert(edge.from_node_id.clone());

                        if let Some(node) = graph.nodes.get(&edge.from_node_id) {
                            if let Some(file_path) = &node.file_path {
                                affected_files.insert(file_path.clone());
                            }
                        }
                    }
                }
            }
        }

        let indirect_count = indirect_dependents.len();
        let score = (direct_count as f64 * 1.0 + indirect_count as f64 * 0.5).min(100.0);

        ImpactReport {
            node_id: target.clone(),
            direct_dependents: direct_count,
            indirect_dependents: indirect_count,
            affected_files: affected_files.len(),
            impact_score: score,
        }
    }
}
