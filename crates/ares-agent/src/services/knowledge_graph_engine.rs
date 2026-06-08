use crate::services::graph_cache::GraphCache;
use ares_core::{AnalysisScope, AresError, GraphStatistics, KnowledgeGraph, ProjectId};
use ares_store::repositories::graph::SqliteGraphRepository;
use std::collections::HashSet;
use std::sync::Arc;

pub struct KnowledgeGraphEngine {
    graph_repo: Arc<SqliteGraphRepository>,
    cache: Arc<GraphCache>,
}

impl KnowledgeGraphEngine {
    pub fn new(graph_repo: Arc<SqliteGraphRepository>, cache: Arc<GraphCache>) -> Self {
        Self { graph_repo, cache }
    }

    pub fn build_graph(
        &self,
        project_id: &ProjectId,
        scope: AnalysisScope,
    ) -> Result<KnowledgeGraph, AresError> {
        // Only checking Project scope caching for now
        if scope == AnalysisScope::Project {
            if let Some(cached) = self.cache.get(project_id) {
                return Ok(cached);
            }
        }

        let nodes = self.graph_repo.get_all_nodes(project_id)?;
        let edges = self.graph_repo.get_all_edges(project_id)?;

        let mut kg = KnowledgeGraph {
            nodes,
            edges,
            statistics: GraphStatistics {
                total_nodes: 0,
                total_edges: 0,
                density: 0.0,
                average_degree: 0.0,
                connected_components: 0,
            },
        };

        kg.statistics = self.graph_statistics(&kg);

        if scope == AnalysisScope::Project {
            self.cache.put(project_id, kg.clone());
        }

        Ok(kg)
    }

    pub fn graph_statistics(&self, kg: &KnowledgeGraph) -> GraphStatistics {
        let nodes = kg.nodes.len();
        let edges = kg.edges.len();

        let density = if nodes > 1 {
            (2.0 * edges as f64) / (nodes as f64 * (nodes as f64 - 1.0))
        } else {
            0.0
        };

        let average_degree = if nodes > 0 {
            (2.0 * edges as f64) / (nodes as f64)
        } else {
            0.0
        };

        // Compute connected components using simple BFS/DFS
        let mut adj: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for edge in &kg.edges {
            adj.entry(edge.from_node_id.as_str().to_string())
                .or_default()
                .push(edge.to_node_id.as_str().to_string());
            adj.entry(edge.to_node_id.as_str().to_string())
                .or_default()
                .push(edge.from_node_id.as_str().to_string()); // Undirected for CC
        }

        let mut visited = HashSet::new();
        let mut components = 0;

        for node in &kg.nodes {
            let id = node.id.as_str().to_string();
            if !visited.contains(&id) {
                components += 1;
                let mut queue = vec![id.clone()];
                visited.insert(id.clone());

                while let Some(curr) = queue.pop() {
                    if let Some(neighbors) = adj.get(&curr) {
                        for n in neighbors {
                            if !visited.contains(n) {
                                visited.insert(n.clone());
                                queue.push(n.clone());
                            }
                        }
                    }
                }
            }
        }

        GraphStatistics {
            total_nodes: nodes,
            total_edges: edges,
            density,
            average_degree,
            connected_components: components,
        }
    }
}
