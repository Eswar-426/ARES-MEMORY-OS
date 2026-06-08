use ares_core::{KnowledgeGraph, ProjectId};
use metrics::counter;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct GraphCache {
    /// In-memory storage mapping ProjectId to its cached KnowledgeGraph.
    cache: Arc<RwLock<HashMap<ProjectId, KnowledgeGraph>>>,
}

impl GraphCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, project_id: &ProjectId) -> Option<KnowledgeGraph> {
        let read_guard = self.cache.read().unwrap();
        if let Some(graph) = read_guard.get(project_id) {
            counter!("ares_graph_cache_hits_total").increment(1);
            Some(graph.clone())
        } else {
            counter!("ares_graph_cache_misses_total").increment(1);
            None
        }
    }

    pub fn put(&self, project_id: &ProjectId, graph: KnowledgeGraph) {
        let mut write_guard = self.cache.write().unwrap();
        write_guard.insert(project_id.clone(), graph);
    }

    pub fn invalidate(&self, project_id: &ProjectId) {
        let mut write_guard = self.cache.write().unwrap();
        if write_guard.remove(project_id).is_some() {
            counter!("ares_graph_cache_invalidations_total").increment(1);
        }
    }

    pub fn invalidate_all(&self) {
        let mut write_guard = self.cache.write().unwrap();
        let count = write_guard.len() as u64;
        write_guard.clear();
        counter!("ares_graph_cache_invalidations_total").increment(count);
    }
}

impl Default for GraphCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ares_core::{GraphStatistics, ProjectId};

    fn dummy_graph() -> KnowledgeGraph {
        KnowledgeGraph {
            nodes: vec![],
            edges: vec![],
            statistics: GraphStatistics {
                total_nodes: 0,
                total_edges: 0,
                density: 0.0,
                average_degree: 0.0,
                connected_components: 0,
            },
        }
    }

    #[test]
    fn cache_put_and_get() {
        let cache = GraphCache::new();
        let pid = ProjectId::from("p1");
        assert!(cache.get(&pid).is_none());

        cache.put(&pid, dummy_graph());
        assert!(cache.get(&pid).is_some());
    }

    #[test]
    fn cache_invalidation() {
        let cache = GraphCache::new();
        let pid = ProjectId::from("p1");
        cache.put(&pid, dummy_graph());
        cache.invalidate(&pid);
        assert!(cache.get(&pid).is_none());
    }
}
