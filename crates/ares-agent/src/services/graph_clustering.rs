use ares_core::{KnowledgeCluster, KnowledgeGraph, NodeId};
use std::collections::{HashMap, HashSet};

pub struct GraphClusteringEngine {}

impl GraphClusteringEngine {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for GraphClusteringEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphClusteringEngine {
    pub fn discover_clusters(&self, kg: &KnowledgeGraph) -> Vec<KnowledgeCluster> {
        // Simple Louvain-like heuristic or connected components for now.
        // The requirements ask for community detection and semantic clusters with cohesion/coupling metrics.
        // We will build an undirected adjacency list with edge weights.
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        let mut node_map = HashMap::new();

        for node in &kg.nodes {
            node_map.insert(node.id.as_str().to_string(), node.clone());
        }

        for edge in &kg.edges {
            let from = edge.from_node_id.as_str().to_string();
            let to = edge.to_node_id.as_str().to_string();
            adj.entry(from.clone()).or_default().push(to.clone());
            adj.entry(to).or_default().push(from); // undirected for clustering
        }

        let mut visited = HashSet::new();
        let mut clusters = Vec::new();

        for node in &kg.nodes {
            let id = node.id.as_str().to_string();
            if !visited.contains(&id) {
                let mut cluster_nodes = Vec::new();
                let mut queue = vec![id.clone()];
                visited.insert(id.clone());

                while let Some(curr) = queue.pop() {
                    cluster_nodes.push(curr.clone());
                    if let Some(neighbors) = adj.get(&curr) {
                        for n in neighbors {
                            if !visited.contains(n) {
                                visited.insert(n.clone());
                                queue.push(n.clone());
                            }
                        }
                    }
                }

                // Calculate cohesion and coupling
                let mut internal_edges = 0;
                let mut external_edges = 0;
                let cluster_set: HashSet<String> = cluster_nodes.iter().cloned().collect();

                for c_node in &cluster_nodes {
                    if let Some(neighbors) = adj.get(c_node) {
                        for n in neighbors {
                            if cluster_set.contains(n) {
                                internal_edges += 1;
                            } else {
                                external_edges += 1;
                            }
                        }
                    }
                }

                // Each internal edge is counted twice in the undirected adj
                internal_edges /= 2;

                let possible_internal_edges =
                    (cluster_nodes.len() * (cluster_nodes.len().saturating_sub(1))) / 2;
                let cohesion = if possible_internal_edges > 0 {
                    internal_edges as f64 / possible_internal_edges as f64
                } else {
                    1.0 // single node cluster is perfectly cohesive
                };

                let total_possible_external =
                    cluster_nodes.len() * (kg.nodes.len() - cluster_nodes.len());
                let coupling = if total_possible_external > 0 {
                    external_edges as f64 / total_possible_external as f64
                } else {
                    0.0
                };

                let nodes: Vec<NodeId> = cluster_nodes
                    .iter()
                    .map(|s| NodeId::from(s.clone()))
                    .collect();

                let main_concept = if !node_map.is_empty() {
                    node_map
                        .get(&cluster_nodes[0])
                        .map(|n| n.label.clone())
                        .unwrap_or_else(|| "Unknown".to_string())
                } else {
                    "Unknown".to_string()
                };

                clusters.push(KnowledgeCluster {
                    id: format!("cluster_{}", clusters.len()),
                    name: format!("Cluster centered around {}", main_concept),
                    nodes,
                    cohesion,
                    coupling,
                });
            }
        }

        clusters
    }
}
