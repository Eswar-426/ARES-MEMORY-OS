use anyhow::Result;
use ares_core::{GraphEdge, GraphNode, NodeId, ProjectId};
use ares_store::repositories::graph::SqliteGraphRepository;
use ares_store::Store;
use std::collections::HashMap;

pub struct ReasoningGraph {
    pub nodes: HashMap<NodeId, GraphNode>,
    pub outgoing: HashMap<NodeId, Vec<GraphEdge>>,
    pub incoming: HashMap<NodeId, Vec<GraphEdge>>,
}

impl ReasoningGraph {
    pub fn build(store: &Store, project_id: &ProjectId) -> Result<Self> {
        let repo = SqliteGraphRepository::new(store.clone());
        let nodes_vec = repo.get_all_nodes(project_id)?;
        let edges_vec = repo.get_all_edges(project_id)?;

        let mut nodes = HashMap::new();
        for node in nodes_vec {
            nodes.insert(node.id.clone(), node);
        }

        let mut outgoing: HashMap<NodeId, Vec<GraphEdge>> = HashMap::new();
        let mut incoming: HashMap<NodeId, Vec<GraphEdge>> = HashMap::new();

        for edge in edges_vec {
            outgoing.entry(edge.from_node_id.clone()).or_default().push(edge.clone());
            incoming.entry(edge.to_node_id.clone()).or_default().push(edge);
        }

        Ok(Self {
            nodes,
            outgoing,
            incoming,
        })
    }

    pub fn to_petgraph(
        &self,
    ) -> (
        petgraph::graph::DiGraph<NodeId, ()>,
        HashMap<NodeId, petgraph::graph::NodeIndex>,
    ) {
        let mut graph = petgraph::graph::DiGraph::new();
        let mut node_indices = HashMap::new();

        for node_id in self.nodes.keys() {
            let idx = graph.add_node(node_id.clone());
            node_indices.insert(node_id.clone(), idx);
        }

        for (from_id, edges) in &self.outgoing {
            if let Some(from_idx) = node_indices.get(from_id) {
                for edge in edges {
                    if let Some(to_idx) = node_indices.get(&edge.to_node_id) {
                        graph.add_edge(*from_idx, *to_idx, ());
                    }
                }
            }
        }

        (graph, node_indices)
    }
}
