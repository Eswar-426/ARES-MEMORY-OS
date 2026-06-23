use ares_core::types::node::NodeType;
use ares_core::AresError;
use ares_core::NodeId;
use ares_core::ProjectId;
use ares_store::repositories::graph::SqliteGraphRepository;
use ares_store::Store;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::ownership_engine::{OwnerInfo, OwnershipEngine};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityOwnership {
    pub capability_node_id: String,
    pub capability_name: String,
    pub owner: Option<OwnerInfo>,
}

pub struct CapabilityEngine {
    store: Arc<Store>,
    ownership_engine: OwnershipEngine,
}

impl CapabilityEngine {
    pub fn new(store: Arc<Store>) -> Self {
        let ownership_engine = OwnershipEngine::new(store.clone());
        Self {
            store,
            ownership_engine,
        }
    }

    /// Determines which team or person owns a specific capability (Feature node).
    /// Ownership is derived strictly from the memory hierarchy.
    pub fn get_capability_owner(
        &self,
        capability_node_id: &str,
    ) -> Result<Option<OwnerInfo>, AresError> {
        let graph = SqliteGraphRepository::new((*self.store).clone());
        let capability_id_obj = NodeId::from(capability_node_id.to_string());
        let node_opt = graph.get_node(&capability_id_obj)?;
        if let Some(node) = node_opt {
            if node.node_type == NodeType::Feature {
                // Determine owner using the OwnershipEngine, which correctly walks the graph
                // to find explicit or inherited ownership (Team > Person).
                return self.ownership_engine.resolve_owner(capability_node_id);
            }
        }
        Ok(None)
    }

    /// Scans all capabilities (Feature nodes) and maps them to their owners.
    pub fn map_all_capabilities(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<CapabilityOwnership>, AresError> {
        let mut mapping = Vec::new();

        let graph = SqliteGraphRepository::new((*self.store).clone());
        let all_nodes = graph.get_all_nodes(project_id)?;
        for node in all_nodes {
            if node.node_type == NodeType::Feature {
                let owner = self.get_capability_owner(&node.id.to_string())?;
                mapping.push(CapabilityOwnership {
                    capability_node_id: node.id.to_string(),
                    capability_name: node.label,
                    owner,
                });
            }
        }

        Ok(mapping)
    }
}
