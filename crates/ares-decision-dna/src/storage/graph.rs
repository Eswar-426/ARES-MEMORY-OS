use crate::models::DecisionMemory;
use anyhow::Result;
use ares_core::types::node::NodeType;

pub struct GraphIntegration;

impl GraphIntegration {
    pub fn build_decision_nodes(
        decision: &DecisionMemory,
    ) -> Result<Vec<ares_core::types::node::GraphNode>> {
        // Implementation for mapping DecisionMemory to GraphNodes
        let mut nodes = vec![];

        // Main decision node
        let decision_node = ares_core::types::node::GraphNode {
            id: ares_core::id::NodeId::from(decision.id.to_string()),
            node_type: NodeType::Decision,
            project_id: ares_core::id::ProjectId::from("".to_string()), // In real implementation, pass project_id
            label: decision.title.clone(),
            file_path: None,
            properties: serde_json::json!({
                "context": decision.context,
                "state": decision.state,
                "confidence": decision.confidence,
                "version": decision.version,
            }),
            created_at: decision.created_at.timestamp_millis(),
            updated_at: decision.updated_at.timestamp_millis(),
            deleted_at: None,
        };
        nodes.push(decision_node);

        Ok(nodes)
    }
}
