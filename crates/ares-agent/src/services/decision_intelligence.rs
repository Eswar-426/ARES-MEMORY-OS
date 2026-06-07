use ares_core::{AresError, Decision, DecisionId, EdgeType, GraphEdge, NodeId, ProjectId};
use ares_store::repositories::decision::SqliteDecisionRepository;
use ares_store::repositories::graph::SqliteGraphRepository;
use std::sync::Arc;

pub struct DecisionIntelligenceEngine {
    decision_repo: Arc<SqliteDecisionRepository>,
    graph_repo: Arc<SqliteGraphRepository>,
}

impl DecisionIntelligenceEngine {
    pub fn new(
        decision_repo: Arc<SqliteDecisionRepository>,
        graph_repo: Arc<SqliteGraphRepository>,
    ) -> Self {
        Self {
            decision_repo,
            graph_repo,
        }
    }

    /// Why was this done? Finds `MotivatedBy` or `Caused` edges from this decision to other nodes.
    pub fn why_was_this_done(
        &self,
        _project_id: &ProjectId,
        decision_id: &DecisionId,
    ) -> Result<Vec<(GraphEdge, ares_core::GraphNode)>, AresError> {
        let node_id = NodeId::from(decision_id.as_str());
        // Find outgoing edges of type MotivatedBy or Caused
        let edges = self.graph_repo.get_edges_from(&node_id)?;
        let mut results = Vec::new();

        for edge in edges {
            if edge.edge_type == EdgeType::MotivatedBy || edge.edge_type == EdgeType::Caused {
                if let Some(node) = self.graph_repo.get_node(&edge.to_node_id)? {
                    results.push((edge, node));
                }
            }
        }
        Ok(results)
    }

    /// What replaced this? Finds `Supersedes` edges pointing TO this decision.
    /// Returns the decision that superseded it.
    pub fn what_replaced_this(
        &self,
        _project_id: &ProjectId,
        decision_id: &DecisionId,
    ) -> Result<Option<Decision>, AresError> {
        let node_id = NodeId::from(decision_id.as_str());
        // Incoming edge: `X -> Supersedes -> this_decision`
        let edges = self.graph_repo.get_edges_to(&node_id)?;

        for edge in edges {
            if edge.edge_type == EdgeType::Supersedes {
                // Return the decision that replaces this one
                let new_decision_id = DecisionId::from(edge.from_node_id.as_str());
                if let Some(dec) = self.decision_repo.get_by_id(&new_decision_id)? {
                    return Ok(Some(dec));
                }
            }
        }
        Ok(None)
    }

    /// What evolved from this? Finds `DerivedFrom` edges pointing TO this decision.
    /// (X -> DerivedFrom -> this_decision) meaning X evolved from this.
    pub fn what_evolved_from_this(
        &self,
        _project_id: &ProjectId,
        decision_id: &DecisionId,
    ) -> Result<Vec<Decision>, AresError> {
        let node_id = NodeId::from(decision_id.as_str());
        let edges = self.graph_repo.get_edges_to(&node_id)?;

        let mut results = Vec::new();
        for edge in edges {
            if edge.edge_type == EdgeType::DerivedFrom {
                let evolved_id = DecisionId::from(edge.from_node_id.as_str());
                if let Some(dec) = self.decision_repo.get_by_id(&evolved_id)? {
                    results.push(dec);
                }
            }
        }
        Ok(results)
    }

    /// Retrieves the linear history of supersedence.
    /// Walks backwards and forwards via `Supersedes` edges.
    pub fn decision_history(
        &self,
        _project_id: &ProjectId,
        decision_id: &DecisionId,
    ) -> Result<Vec<Decision>, AresError> {
        let mut history = Vec::new();

        // 1. Walk backward (what did this supersede?)
        let mut current_id = decision_id.clone();
        let mut backward_chain = Vec::new();
        loop {
            let node_id = NodeId::from(current_id.as_str());
            let edges = self.graph_repo.get_edges_from(&node_id)?;
            let mut found = false;
            for edge in edges {
                if edge.edge_type == EdgeType::Supersedes {
                    let prev_id = DecisionId::from(edge.to_node_id.as_str());
                    if let Some(dec) = self.decision_repo.get_by_id(&prev_id)? {
                        backward_chain.push(dec);
                        current_id = prev_id;
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                break;
            }
        }

        backward_chain.reverse();
        history.extend(backward_chain);

        // 2. Add current decision
        if let Some(dec) = self.decision_repo.get_by_id(decision_id)? {
            history.push(dec);
        } else {
            return Err(AresError::not_found("decision", decision_id.as_str()));
        }

        // 3. Walk forward (what superseded this?)
        let mut current_id = decision_id.clone();
        loop {
            let node_id = NodeId::from(current_id.as_str());
            let edges = self.graph_repo.get_edges_to(&node_id)?;
            let mut found = false;
            for edge in edges {
                if edge.edge_type == EdgeType::Supersedes {
                    let next_id = DecisionId::from(edge.from_node_id.as_str());
                    if let Some(dec) = self.decision_repo.get_by_id(&next_id)? {
                        history.push(dec);
                        current_id = next_id;
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                break;
            }
        }

        Ok(history)
    }
}
