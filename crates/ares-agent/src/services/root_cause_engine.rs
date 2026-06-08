use ares_core::{AresError, EvidenceStep, KnowledgeGraph, NodeId};
use std::collections::{HashMap, HashSet};

pub struct RootCauseEngine {}

impl RootCauseEngine {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for RootCauseEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RootCauseEngine {
    pub fn find_root_cause(
        &self,
        kg: &KnowledgeGraph,
        target_node: &NodeId,
    ) -> Result<ares_core::RootCauseAnalysis, AresError> {
        let mut adj_rev: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &kg.edges {
            // For root cause, we trace backwards (from -> to means `to` depends on `from`)
            adj_rev
                .entry(edge.to_node_id.as_str().to_string())
                .or_default()
                .push(edge.from_node_id.as_str().to_string());
        }

        let target_str = target_node.as_str().to_string();
        if !kg.nodes.iter().any(|n| n.id.as_str() == target_str) {
            return Err(AresError::not_found("node", &target_str));
        }

        let mut visited = HashSet::new();
        let mut evidence_chain = Vec::new();

        // Simple DFS to find the deepest node (the "root")
        let root = self.dfs_longest_path(
            &adj_rev,
            &target_str,
            &mut visited,
            &mut evidence_chain,
            1.0,
        );

        // Sort evidence chain by confidence or keep as order of discovery
        let root_id = root.map(NodeId::from);

        Ok(ares_core::RootCauseAnalysis {
            target_node: target_node.clone(),
            root_cause_node: root_id,
            evidence_chain,
        })
    }

    fn dfs_longest_path(
        &self,
        adj: &HashMap<String, Vec<String>>,
        current: &str,
        visited: &mut HashSet<String>,
        evidence_chain: &mut Vec<EvidenceStep>,
        confidence: f64,
    ) -> Option<String> {
        visited.insert(current.to_string());

        let next_nodes = adj.get(current);
        if next_nodes.is_none() || next_nodes.unwrap().is_empty() {
            evidence_chain.push(EvidenceStep {
                description: format!("Node {} is a terminal dependency", current),
                node_id: Some(NodeId::from(current)),
                confidence,
            });
            return Some(current.to_string());
        }

        let mut deepest_root = current.to_string();
        for neighbor in next_nodes.unwrap() {
            if !visited.contains(neighbor) {
                evidence_chain.push(EvidenceStep {
                    description: format!("Tracing backward to dependency {}", neighbor),
                    node_id: Some(NodeId::from(neighbor.as_str())),
                    confidence: confidence * 0.9,
                });
                if let Some(res) =
                    self.dfs_longest_path(adj, neighbor, visited, evidence_chain, confidence * 0.9)
                {
                    deepest_root = res;
                }
            }
        }

        Some(deepest_root)
    }
}
