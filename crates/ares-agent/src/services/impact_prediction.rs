use ares_core::{AresError, EdgeType, ImpactPrediction, KnowledgeGraph, NodeId};
use std::collections::{HashMap, VecDeque};

pub struct ImpactPredictionEngine {}

impl ImpactPredictionEngine {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ImpactPredictionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ImpactPredictionEngine {
    pub fn predict_change_impact(
        &self,
        kg: &KnowledgeGraph,
        target_node: &NodeId,
    ) -> Result<ImpactPrediction, AresError> {
        let mut adj: HashMap<String, Vec<(String, EdgeType)>> = HashMap::new();
        let mut node_map = HashMap::new();

        for node in &kg.nodes {
            node_map.insert(node.id.as_str().to_string(), node.clone());
        }

        for edge in &kg.edges {
            adj.entry(edge.from_node_id.as_str().to_string())
                .or_default()
                .push((edge.to_node_id.as_str().to_string(), edge.edge_type.clone()));
        }

        let target_str = target_node.as_str().to_string();
        if !node_map.contains_key(&target_str) {
            return Err(AresError::not_found("node", &target_str));
        }

        let mut visited = HashMap::new();
        let mut queue = VecDeque::new();

        queue.push_back((target_str.clone(), 0, 1.0_f64));
        visited.insert(target_str.clone(), 1.0_f64);

        let mut affected_modules = Vec::new();
        let mut total_risk: f64 = 0.0;
        let mut blast_radius = 0;

        while let Some((curr, depth, conf)) = queue.pop_front() {
            if depth > 0 {
                blast_radius += 1;
                total_risk += conf;
                if let Some(n) = node_map.get(&curr) {
                    if n.node_type == ares_core::NodeType::Module {
                        affected_modules.push(curr.clone());
                    }
                }
            }

            if depth < 5 {
                if let Some(neighbors) = adj.get(&curr) {
                    for (n, edge_type) in neighbors {
                        let decay: f64 = match edge_type {
                            EdgeType::DependsOn | EdgeType::Calls | EdgeType::References => 0.1,
                            EdgeType::Contradicts | EdgeType::Supersedes => 0.05,
                            _ => 0.2,
                        };
                        let next_conf = (conf - decay).max(0.0_f64);
                        if next_conf > 0.0_f64
                            && (!visited.contains_key(n) || visited[n] < next_conf)
                        {
                            visited.insert(n.clone(), next_conf);
                            queue.push_back((n.clone(), depth + 1, next_conf));
                        }
                    }
                }
            }
        }

        let avg_confidence = if blast_radius > 0 {
            total_risk / (blast_radius as f64)
        } else {
            1.0
        };

        let risk_score = 1.0 - std::f64::consts::E.powf(-(blast_radius as f64) / 10.0);

        Ok(ImpactPrediction {
            target_node: target_node.clone(),
            blast_radius,
            risk_score,
            affected_modules,
            confidence: avg_confidence,
        })
    }
}
