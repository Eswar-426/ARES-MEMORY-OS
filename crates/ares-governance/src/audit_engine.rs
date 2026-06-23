use ares_core::types::evolution::EvolutionEvent;
use ares_core::types::node::NodeType;
use ares_core::AresError;
use ares_core::NodeId;
use ares_core::ProjectId;
use ares_store::repositories::graph::SqliteGraphRepository;
use ares_store::Store;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrail {
    pub node_id: String,
    pub node_type: NodeType,
    pub events: Vec<EvolutionEvent>,
}

pub struct AuditEngine {
    store: Arc<Store>,
}

impl AuditEngine {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    /// Retrieves the complete governance audit trail for a specific memory node.
    pub fn get_audit_trail(
        &self,
        node_id: &str,
        project_id: &ProjectId,
    ) -> Result<Option<AuditTrail>, AresError> {
        let graph = SqliteGraphRepository::new((*self.store).clone());
        let node_id_obj = NodeId::from(node_id.to_string());
        let node_opt = graph.get_node(&node_id_obj)?;
        if let Some(node) = node_opt {
            let mut events = Vec::new();

            // We look for EvolutionEvent nodes that target this node_id
            let all_nodes = graph.get_all_nodes(project_id)?;
            for ev_node in all_nodes {
                if ev_node.node_type == NodeType::EvolutionEvent {
                    if let Ok(ev) =
                        serde_json::from_value::<EvolutionEvent>(ev_node.properties.clone())
                    {
                        if ev.target_node.to_string() == node_id {
                            events.push(ev);
                        }
                    }
                }
            }

            // Sort chronologically
            events.sort_by_key(|e| e.occurred_at);

            return Ok(Some(AuditTrail {
                node_id: node.id.to_string(),
                node_type: node.node_type,
                events,
            }));
        }

        Ok(None)
    }
}
