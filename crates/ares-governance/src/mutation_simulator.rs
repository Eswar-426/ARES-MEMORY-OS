use ares_core::{AresError, EdgeDirection, EdgeType, NodeId, ProjectId};
use ares_core::types::node::{GraphEdge, GraphNode};
use crate::compliance_engine::GraphProvider;

#[derive(Clone)]
pub enum GraphMutation {
    AddNode(GraphNode),
    AddEdge(GraphEdge),
}

pub struct VirtualGraphProvider {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

impl VirtualGraphProvider {
    pub fn new(mut base_nodes: Vec<GraphNode>, mut base_edges: Vec<GraphEdge>, mutation: GraphMutation) -> Self {
        match mutation {
            GraphMutation::AddNode(n) => base_nodes.push(n),
            GraphMutation::AddEdge(e) => base_edges.push(e),
        }
        
        Self {
            nodes: base_nodes,
            edges: base_edges,
        }
    }
}

impl GraphProvider for VirtualGraphProvider {
    fn get_node(&self, id: &NodeId) -> Result<Option<GraphNode>, AresError> {
        Ok(self.nodes.iter().find(|n| n.id == *id).cloned())
    }

    fn get_neighbors(
        &self,
        id: &NodeId,
        direction: EdgeDirection,
        edge_types: &[EdgeType],
    ) -> Result<Vec<GraphNode>, AresError> {
        let mut neighbors = Vec::new();
        
        for edge in &self.edges {
            if !edge_types.contains(&edge.edge_type) {
                continue;
            }
            
            match direction {
                EdgeDirection::Outgoing => {
                    if edge.from_node_id == *id {
                        if let Some(n) = self.get_node(&edge.to_node_id)? {
                            neighbors.push(n);
                        }
                    }
                }
                EdgeDirection::Incoming => {
                    if edge.to_node_id == *id {
                        if let Some(n) = self.get_node(&edge.from_node_id)? {
                            neighbors.push(n);
                        }
                    }
                }
                EdgeDirection::Both => {
                    if edge.from_node_id == *id {
                        if let Some(n) = self.get_node(&edge.to_node_id)? {
                            neighbors.push(n);
                        }
                    } else if edge.to_node_id == *id {
                        if let Some(n) = self.get_node(&edge.from_node_id)? {
                            neighbors.push(n);
                        }
                    }
                }
            }
        }
        
        Ok(neighbors)
    }

    fn get_all_nodes(&self, project_id: &ProjectId) -> Result<Vec<GraphNode>, AresError> {
        Ok(self.nodes.iter().filter(|n| n.project_id == *project_id).cloned().collect())
    }
}
