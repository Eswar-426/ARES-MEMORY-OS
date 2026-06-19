use crate::{EdgeProvider, TraceTargetType, TraceabilityEdge, TraceabilityGraph};
use ares_core::AresError;

pub struct TestEdgeProvider {
    pub edges: Vec<TraceabilityEdge>,
}

impl EdgeProvider for TestEdgeProvider {
    fn edges(&self) -> Result<Vec<TraceabilityEdge>, AresError> {
        Ok(self.edges.clone())
    }
}

pub struct TestGraphBuilder {
    edges: Vec<TraceabilityEdge>,
}

impl TestGraphBuilder {
    pub fn new() -> Self {
        Self { edges: Vec::new() }
    }

    pub fn requirement(self, id: &str) -> Self {
        // Just defining a node implicitly by edges or we could add a self edge, 
        // but typically a node is added via an edge. 
        self
    }

    pub fn decision(self, id: &str) -> Self {
        self
    }

    pub fn code(self, id: &str) -> Self {
        self
    }

    pub fn test(self, id: &str) -> Self {
        self
    }

    pub fn metric(self, id: &str) -> Self {
        self
    }

    pub fn link(mut self, source: &str, target: &str, target_type: TraceTargetType) -> Self {
        self.edges.push(TraceabilityEdge {
            source_id: source.to_string(),
            target_id: target.to_string(),
            target_type,
            relationship: "Validates".to_string(), // generic relationship for tests
        });
        self
    }

    pub fn link_rel(mut self, source: &str, target: &str, target_type: TraceTargetType, relationship: &str) -> Self {
        self.edges.push(TraceabilityEdge {
            source_id: source.to_string(),
            target_id: target.to_string(),
            target_type,
            relationship: relationship.to_string(),
        });
        self
    }

    pub fn build(self) -> TraceabilityGraph {
        let mut graph = TraceabilityGraph::new();
        graph.add_provider(Box::new(TestEdgeProvider { edges: self.edges }));
        graph
    }
}
