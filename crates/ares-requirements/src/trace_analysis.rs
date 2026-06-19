use ares_traceability::{TraceabilityGraph, TraceTargetType};

pub struct TraceAnalysisEngine<'a> {
    pub graph: &'a TraceabilityGraph,
}

impl<'a> TraceAnalysisEngine<'a> {
    pub fn new(graph: &'a TraceabilityGraph) -> Self {
        Self { graph }
    }

    pub fn get_downstream(&self, node_id: &str, target_type: TraceTargetType) -> Vec<String> {
        if let Ok(nodes) = self.graph.find_downstream(node_id) {
            nodes
                .into_iter()
                .filter(|n| n.node_type == target_type)
                .map(|n| n.id)
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_upstream(&self, node_id: &str, target_type: TraceTargetType) -> Vec<String> {
        if let Ok(nodes) = self.graph.find_upstream(node_id) {
            nodes
                .into_iter()
                .filter(|n| n.node_type == target_type)
                .map(|n| n.id)
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn has_decision(&self, req_id: &str) -> bool {
        !self.get_downstream(req_id, TraceTargetType::Decision).is_empty()
    }

    pub fn has_implementation(&self, req_id: &str) -> bool {
        !self.get_downstream(req_id, TraceTargetType::Code).is_empty()
    }

    pub fn has_test(&self, req_id: &str) -> bool {
        !self.get_downstream(req_id, TraceTargetType::Test).is_empty()
    }

    pub fn has_runtime_metric(&self, req_id: &str) -> bool {
        !self.get_downstream(req_id, TraceTargetType::RuntimeMetric).is_empty()
    }

    pub fn get_downstream_all(&self, node_id: &str) -> Vec<ares_traceability::TraceNode> {
        self.graph.find_downstream(node_id).unwrap_or_default()
    }

    pub fn get_upstream_all(&self, node_id: &str) -> Vec<ares_traceability::TraceNode> {
        self.graph.find_upstream(node_id).unwrap_or_default()
    }
}
