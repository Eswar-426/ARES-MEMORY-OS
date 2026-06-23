use ares_core::types::node::{GraphNode, NodeType};
use ares_core::{AresError, EdgeDirection, EdgeType, NodeId, ProjectId};
use ares_store::repositories::graph::SqliteGraphRepository;
use std::sync::Arc;
use ares_store::Store;

pub struct MemoryRetrievalEngine {
    store: Arc<Store>,
}

impl MemoryRetrievalEngine {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    /// Retrieve a node by its ID
    pub fn get_node(&self, id: &str) -> Result<Option<GraphNode>, AresError> {
        let graph = SqliteGraphRepository::new((*self.store).clone());
        let node_id = NodeId::from(id.to_string());
        graph.get_node(&node_id)
    }

    /// Retrieve the immediate graph neighborhood
    pub fn get_neighborhood(
        &self,
        id: &str,
        direction: EdgeDirection,
        edge_types: &[EdgeType],
    ) -> Result<Vec<GraphNode>, AresError> {
        let graph = SqliteGraphRepository::new((*self.store).clone());
        let node_id = NodeId::from(id.to_string());
        graph.get_neighbors(&node_id, direction, edge_types)
    }

    /// Retrieve all nodes of a specific type in a project
    pub fn find_by_type(
        &self,
        project_id: &ProjectId,
        node_type: NodeType,
    ) -> Result<Vec<GraphNode>, AresError> {
        let graph = SqliteGraphRepository::new((*self.store).clone());
        // Since we only have get_all_nodes and list_nodes_paginated, we can filter get_all_nodes
        // or use list_nodes_paginated. For now, filter get_all_nodes.
        let all_nodes = graph.get_all_nodes(project_id)?;
        Ok(all_nodes
            .into_iter()
            .filter(|n| n.node_type == node_type)
            .collect())
    }

    /// Retrieve nodes explicitly owned by the given owner ID
    pub fn find_by_owner(
        &self,
        project_id: &ProjectId,
        owner_id: &str,
    ) -> Result<Vec<GraphNode>, AresError> {
        let graph = SqliteGraphRepository::new((*self.store).clone());
        let owner_node_id = NodeId::from(owner_id.to_string());
        
        // Find nodes where owner_id is the target of an OwnedBy edge,
        // or the source of an Owns edge.
        let mut owned_nodes = Vec::new();
        
        let owns_edges = graph.get_edges_from(&owner_node_id)?;
        for edge in owns_edges {
            if edge.edge_type == EdgeType::Owns {
                if let Some(n) = graph.get_node(&edge.to_node_id)? {
                    if &n.project_id == project_id {
                        owned_nodes.push(n);
                    }
                }
            }
        }

        let owned_by_edges = graph.get_edges_to(&owner_node_id)?;
        for edge in owned_by_edges {
            if edge.edge_type == EdgeType::OwnedBy {
                if let Some(n) = graph.get_node(&edge.from_node_id)? {
                    if &n.project_id == project_id && !owned_nodes.iter().any(|existing| existing.id == n.id) {
                        owned_nodes.push(n);
                    }
                }
            }
        }
        
        Ok(owned_nodes)
    }

    pub fn get_all_edges_from(&self, id: &str) -> Result<Vec<ares_core::types::node::GraphEdge>, AresError> {
        let graph = SqliteGraphRepository::new((*self.store).clone());
        let node_id = NodeId::from(id.to_string());
        graph.get_edges_from(&node_id)
    }

    pub fn get_all_edges_to(&self, id: &str) -> Result<Vec<ares_core::types::node::GraphEdge>, AresError> {
        let graph = SqliteGraphRepository::new((*self.store).clone());
        let node_id = NodeId::from(id.to_string());
        graph.get_edges_to(&node_id)
    }
}
