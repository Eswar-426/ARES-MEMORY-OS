use ares_core::{AresError, NodeId, EdgeDirection, EdgeType, types::node::{GraphNode, NodeType}};
use ares_retrieval::memory_retrieval_engine::MemoryRetrievalEngine;

pub struct DecisionLineageEngine<'a> {
    retrieval_engine: &'a MemoryRetrievalEngine,
}

impl<'a> DecisionLineageEngine<'a> {
    pub fn new(retrieval_engine: &'a MemoryRetrievalEngine) -> Self {
        Self { retrieval_engine }
    }

    pub fn get_originating_decision(&self, file_id: &NodeId) -> Result<Option<GraphNode>, AresError> {
        let lineage = self.get_decision_lineage(file_id)?;
        Ok(lineage.into_iter().find(|n| n.node_type == NodeType::Decision))
    }

    pub fn get_originating_requirement(&self, file_id: &NodeId) -> Result<Option<GraphNode>, AresError> {
        let lineage = self.get_decision_lineage(file_id)?;
        Ok(lineage.into_iter().find(|n| n.node_type == NodeType::Requirement))
    }

    pub fn get_decision_lineage(&self, node_id: &NodeId) -> Result<Vec<GraphNode>, AresError> {
        let mut lineage = Vec::new();
        let initial_node = self.retrieval_engine.get_node(&node_id.to_string())?
            .ok_or_else(|| AresError::NotFound { resource_type: "Node".into(), id: node_id.to_string() })?;
        
        let mut current_nodes = vec![initial_node];

        while !current_nodes.is_empty() {
            let mut next_nodes = Vec::new();
            for node in current_nodes {
                lineage.push(node.clone());

                let upstream = self.retrieval_engine.get_neighborhood(
                    &node.id.to_string(),
                    EdgeDirection::Incoming,
                    &[EdgeType::Drives, EdgeType::Implements],
                )?;
                next_nodes.extend(upstream);
            }
            current_nodes = next_nodes;
        }

        Ok(lineage)
    }
}
