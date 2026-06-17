use ares_core::{Decision, GraphEdge, GraphNode, Memory, Project};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBudget {
    pub max_memories: usize,
    pub max_decisions: usize,
    pub max_graph_nodes: usize,
    pub max_graph_edges: usize,
    pub max_total_tokens: usize,
}

impl Default for ContextBudget {
    fn default() -> Self {
        Self {
            max_memories: 20,
            max_decisions: 10,
            max_graph_nodes: 50,
            max_graph_edges: 100,
            max_total_tokens: 32_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(utoipa::ToSchema)]
pub struct ContextSnapshot {
    pub memories: Vec<Memory>,
    pub decisions: Vec<Decision>,
    pub graph_nodes: Vec<GraphNode>,
    pub graph_edges: Vec<GraphEdge>,
    pub estimated_tokens: usize,
}

pub struct ContextBuilder;

impl ContextBuilder {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextBuilder {
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        &self,
        project: &Project,
        query: &str,
        mut memories: Vec<Memory>,
        mut decisions: Vec<Decision>,
        mut graph_nodes: Vec<GraphNode>,
        mut graph_edges: Vec<GraphEdge>,
        budget: ContextBudget,
    ) -> ContextSnapshot {
        // We assume memories, decisions, nodes are already sorted by relevance
        // from the ranking pipeline. We just truncate according to budget and estimate tokens.

        let mut snapshot_memories = Vec::new();
        let mut snapshot_decisions = Vec::new();
        let mut snapshot_nodes = Vec::new();
        let mut snapshot_edges = Vec::new();
        let mut total_tokens = self.estimate_base_tokens(project, query);

        for mem in memories.drain(..) {
            if snapshot_memories.len() >= budget.max_memories {
                break;
            }
            let tokens = self.estimate_memory_tokens(&mem);
            if total_tokens + tokens > budget.max_total_tokens {
                break;
            }
            total_tokens += tokens;
            snapshot_memories.push(mem);
        }

        for dec in decisions.drain(..) {
            if snapshot_decisions.len() >= budget.max_decisions {
                break;
            }
            let tokens = self.estimate_decision_tokens(&dec);
            if total_tokens + tokens > budget.max_total_tokens {
                break;
            }
            total_tokens += tokens;
            snapshot_decisions.push(dec);
        }

        for node in graph_nodes.drain(..) {
            if snapshot_nodes.len() >= budget.max_graph_nodes {
                break;
            }
            let tokens = self.estimate_node_tokens(&node);
            if total_tokens + tokens > budget.max_total_tokens {
                break;
            }
            total_tokens += tokens;
            snapshot_nodes.push(node);
        }

        for edge in graph_edges.drain(..) {
            if snapshot_edges.len() >= budget.max_graph_edges {
                break;
            }
            let tokens = self.estimate_edge_tokens(&edge);
            if total_tokens + tokens > budget.max_total_tokens {
                break;
            }
            total_tokens += tokens;
            snapshot_edges.push(edge);
        }

        ContextSnapshot {
            memories: snapshot_memories,
            decisions: snapshot_decisions,
            graph_nodes: snapshot_nodes,
            graph_edges: snapshot_edges,
            estimated_tokens: total_tokens,
        }
    }

    // Crude token estimation based on character count / 4
    fn estimate_base_tokens(&self, project: &Project, query: &str) -> usize {
        (project.name.len() + project.description.len() + query.len()) / 4 + 10
    }

    fn estimate_memory_tokens(&self, memory: &Memory) -> usize {
        let content_len = memory.content.to_string().len();
        (memory.title.len() + content_len) / 4 + 10
    }

    fn estimate_decision_tokens(&self, decision: &Decision) -> usize {
        let text_len = decision.title.len() + decision.reason.len() + decision.decision_text.len();
        text_len / 4 + 20
    }

    fn estimate_node_tokens(&self, node: &GraphNode) -> usize {
        let props_len = node.properties.to_string().len();
        (node.label.len() + props_len) / 4 + 10
    }

    fn estimate_edge_tokens(&self, _edge: &GraphEdge) -> usize {
        10 // Edges are short IDs and types
    }
}
