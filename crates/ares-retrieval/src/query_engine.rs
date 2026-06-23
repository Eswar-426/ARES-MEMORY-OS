use crate::memory_retrieval_engine::MemoryRetrievalEngine;
use ares_core::types::node::{GraphNode, NodeType};
use ares_core::{AresError, ProjectId};

pub struct QueryEngine<'a> {
    retrieval_engine: &'a MemoryRetrievalEngine,
}

impl<'a> QueryEngine<'a> {
    pub fn new(retrieval_engine: &'a MemoryRetrievalEngine) -> Self {
        Self { retrieval_engine }
    }

    /// Find all decisions related to a specific capability (Feature)
    pub fn find_decisions_for_capability(
        &self,
        project_id: &ProjectId,
        capability_id: &str,
    ) -> Result<Vec<GraphNode>, AresError> {
        let mut decisions = Vec::new();

        // Capabilities are Feature nodes. They are usually driven by requirements or decisions.
        // We'll traverse Incoming Drives/Contains to find requirements/decisions.
        // But the simplest approach is just returning decisions that relate to this capability.
        // For a full implementation, we might do a Graph Traverse.
        let _nodes = self
            .retrieval_engine
            .find_by_type(project_id, NodeType::Decision)?;

        // This is a naive implementation; assuming capability ID is referenced in properties or edges.
        // For deterministic retrieval, we would traverse edges.
        let capability = self.retrieval_engine.get_node(capability_id)?;
        if let Some(cap) = capability {
            // Find edges connecting them
            let incoming_edges = self.retrieval_engine.get_neighborhood(
                &cap.id.to_string(),
                ares_core::EdgeDirection::Incoming,
                &[ares_core::EdgeType::Drives, ares_core::EdgeType::Contains],
            )?;
            for node in incoming_edges {
                if node.node_type == NodeType::Decision {
                    decisions.push(node);
                }
            }
        }

        Ok(decisions)
    }

    /// Find all requirements owned by a specific team/owner
    pub fn find_requirements_by_owner(
        &self,
        project_id: &ProjectId,
        owner_id: &str,
    ) -> Result<Vec<GraphNode>, AresError> {
        let owned = self.retrieval_engine.find_by_owner(project_id, owner_id)?;
        Ok(owned
            .into_iter()
            .filter(|n| n.node_type == NodeType::Requirement)
            .collect())
    }

    /// Find orphaned architecture (Architecture nodes missing Code or Requirement)
    pub fn find_orphaned_architecture(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<GraphNode>, AresError> {
        let arch_nodes = self
            .retrieval_engine
            .find_by_type(project_id, NodeType::Architecture)?;
        let mut orphans = Vec::new();

        for arch in arch_nodes {
            // Check if it has incoming Drives from Decision/Req
            let incoming = self.retrieval_engine.get_neighborhood(
                &arch.id.to_string(),
                ares_core::EdgeDirection::Incoming,
                &[ares_core::EdgeType::Drives, ares_core::EdgeType::Implements],
            )?;
            // Check if it has outgoing Drives/Contains to Code
            let outgoing = self.retrieval_engine.get_neighborhood(
                &arch.id.to_string(),
                ares_core::EdgeDirection::Outgoing,
                &[ares_core::EdgeType::Drives, ares_core::EdgeType::Contains],
            )?;

            let has_upstream = !incoming.is_empty();
            let has_downstream = !outgoing.is_empty();

            if !has_upstream || !has_downstream {
                orphans.push(arch);
            }
        }

        Ok(orphans)
    }

    /// Find drifted capabilities
    pub fn find_drifted_capabilities(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<GraphNode>, AresError> {
        // Drift is typically identified by the Evolution engine and stored as property "has_drift: true"
        // or via EvolutionEvent. We'll search properties for now.
        let features = self
            .retrieval_engine
            .find_by_type(project_id, NodeType::Feature)?;
        let mut drifted = Vec::new();

        for feat in features {
            if feat
                .properties
                .get("has_drift")
                .is_some_and(|v| v.as_bool().unwrap_or(false))
            {
                drifted.push(feat);
            }
        }

        Ok(drifted)
    }
}
