use ares_core::types::node::{EdgeType, NodeType};
use ares_core::AresError;
use ares_core::NodeId;
use ares_store::repositories::graph::SqliteGraphRepository;
use ares_store::Store;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OwnerInfo {
    pub owner_id: String,
    pub owner_type: NodeType, // Team or Person
    pub is_explicit: bool,
    pub inherited_from_node_id: Option<String>,
}

pub struct OwnershipEngine {
    store: Arc<Store>,
}

impl OwnershipEngine {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    /// Resolves the authoritative owner for a given node.
    /// Rules:
    /// 1. Explicit overrides inherited.
    /// 2. Team ownership is authoritative over Person.
    pub fn resolve_owner(&self, node_id: &str) -> Result<Option<OwnerInfo>, AresError> {
        // 1. Check explicit owners
        if let Some(explicit_owner) = self.get_explicit_owner(node_id)? {
            return Ok(Some(explicit_owner));
        }

        // 2. Check inherited owners by walking UP the memory hierarchy
        // Memory hierarchy flows DOWN via `Drives` and `Contains`.
        // So to inherit, we must walk UP via Incoming `Drives` or Incoming `Contains`.
        let inherited = self.walk_up_for_owner(node_id)?;
        Ok(inherited)
    }

    fn get_explicit_owner(&self, node_id: &str) -> Result<Option<OwnerInfo>, AresError> {
        // We look for edges where:
        // owner -> Owns -> node
        // OR node -> OwnedBy -> owner
        let mut potential_owners = Vec::new();

        let graph = SqliteGraphRepository::new((*self.store).clone());
        let node_id_obj = NodeId::from(node_id.to_string());
        let incoming = graph.get_edges_to(&node_id_obj)?;
        for edge in incoming {
            if edge.edge_type == EdgeType::Owns {
                potential_owners.push(edge.from_node_id.to_string());
            }
        }

        let outgoing = graph.get_edges_from(&node_id_obj)?;
        for edge in outgoing {
            if edge.edge_type == EdgeType::OwnedBy {
                potential_owners.push(edge.to_node_id.to_string());
            }
        }

        self.select_authoritative_owner(potential_owners, true, None)
    }

    fn walk_up_for_owner(&self, node_id: &str) -> Result<Option<OwnerInfo>, AresError> {
        let mut current_id = node_id.to_string();
        let mut visited = std::collections::HashSet::new();

        while !visited.contains(&current_id) {
            visited.insert(current_id.clone());

            // Look for parents: Incoming `Drives` or Incoming `Contains` or Incoming `DependsOn`
            let graph = SqliteGraphRepository::new((*self.store).clone());
            let current_id_obj = NodeId::from(current_id.clone());
            let incoming = graph.get_edges_to(&current_id_obj)?;
            let mut next_parent = None;

            for edge in incoming {
                if edge.edge_type == EdgeType::Drives || edge.edge_type == EdgeType::Contains {
                    next_parent = Some(edge.from_node_id.to_string());
                    break;
                }
            }

            if let Some(parent_id) = next_parent {
                if let Some(mut inherited_owner) = self.get_explicit_owner(&parent_id)? {
                    inherited_owner.is_explicit = false;
                    inherited_owner.inherited_from_node_id = Some(parent_id.to_string());
                    return Ok(Some(inherited_owner));
                }
                current_id = parent_id.to_string();
            } else {
                break;
            }
        }

        Ok(None)
    }

    fn select_authoritative_owner(
        &self,
        owner_ids: Vec<String>,
        is_explicit: bool,
        inherited_from: Option<String>,
    ) -> Result<Option<OwnerInfo>, AresError> {
        if owner_ids.is_empty() {
            return Ok(None);
        }

        let mut team_owner = None;
        let mut person_owner = None;

        for oid in owner_ids {
            let graph = SqliteGraphRepository::new((*self.store).clone());
            let oid_obj = NodeId::from(oid.clone());
            let node = graph.get_node(&oid_obj)?;
            if let Some(n) = node {
                if n.node_type == NodeType::Team {
                    team_owner = Some(oid);
                } else if n.node_type == NodeType::Person {
                    person_owner = Some(oid);
                }
            }
        }

        // Team is authoritative
        if let Some(tid) = team_owner {
            return Ok(Some(OwnerInfo {
                owner_id: tid,
                owner_type: NodeType::Team,
                is_explicit,
                inherited_from_node_id: inherited_from,
            }));
        }

        if let Some(pid) = person_owner {
            return Ok(Some(OwnerInfo {
                owner_id: pid,
                owner_type: NodeType::Person,
                is_explicit,
                inherited_from_node_id: inherited_from,
            }));
        }

        Ok(None)
    }
}
