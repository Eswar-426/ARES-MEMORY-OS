use ares_core::types::node::{EdgeType, NodeType};
use ares_core::AresError;
use ares_core::NodeId;
use ares_store::repositories::graph::SqliteGraphRepository;
use ares_store::Store;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalInfo {
    pub approved_by_node_id: String,
    pub is_reviewer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalLineage {
    pub node_id: String,
    pub node_type: NodeType,
    pub approvals: Vec<ApprovalInfo>,
}

pub struct ApprovalGraphEngine {
    store: Arc<Store>,
}

impl ApprovalGraphEngine {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    /// Traces the approval chain of a specific Requirement or Decision node.
    pub fn trace_approvals(&self, node_id: &str) -> Result<Option<ApprovalLineage>, AresError> {
        let graph = SqliteGraphRepository::new((*self.store).clone());
        let node_id_obj = NodeId::from(node_id.to_string());
        let node_opt = graph.get_node(&node_id_obj)?;
        if let Some(node) = node_opt {
            // Approvals are strictly restricted to Requirement and Decision memory layers.
            if node.node_type != NodeType::Requirement && node.node_type != NodeType::Decision {
                return Ok(None);
            }

            let mut approvals = Vec::new();

            // Look for ApprovedBy and ReviewedBy edges (Outgoing from Node to Person/Team)
            let outgoing_edges = graph.get_edges_from(&node_id_obj)?;

            for edge in outgoing_edges {
                if edge.edge_type == EdgeType::ApprovedBy {
                    approvals.push(ApprovalInfo {
                        approved_by_node_id: edge.to_node_id.to_string(),
                        is_reviewer: false,
                    });
                } else if edge.edge_type == EdgeType::ReviewedBy {
                    approvals.push(ApprovalInfo {
                        approved_by_node_id: edge.to_node_id.to_string(),
                        is_reviewer: true,
                    });
                }
            }

            return Ok(Some(ApprovalLineage {
                node_id: node.id.to_string(),
                node_type: node.node_type,
                approvals,
            }));
        }

        Ok(None)
    }

    /// Verifies if a given node is considered "fully approved".
    /// Fully approved means it has at least one explicit approval.
    pub fn is_approved(&self, node_id: &str) -> Result<bool, AresError> {
        if let Some(lineage) = self.trace_approvals(node_id)? {
            // True if there is at least one direct ApprovedBy relationship.
            return Ok(lineage.approvals.iter().any(|a| !a.is_reviewer));
        }
        Ok(false)
    }
}
