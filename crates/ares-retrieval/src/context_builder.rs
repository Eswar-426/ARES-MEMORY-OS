use crate::memory_retrieval_engine::MemoryRetrievalEngine;
use crate::models::MemoryContextPack;
use ares_core::types::node::NodeType;
use ares_core::{AresError, EdgeDirection, EdgeType};

pub struct ContextBuilder<'a> {
    retrieval_engine: &'a MemoryRetrievalEngine,
}

impl<'a> ContextBuilder<'a> {
    pub fn new(retrieval_engine: &'a MemoryRetrievalEngine) -> Self {
        Self { retrieval_engine }
    }

    /// Builds a comprehensive memory context pack starting from a specific focal node
    pub fn build_context_pack(&self, focal_node_id: &str) -> Result<MemoryContextPack, AresError> {
        let node = match self.retrieval_engine.get_node(focal_node_id)? {
            Some(n) => n,
            None => return Err(AresError::not_found("node", focal_node_id)),
        };

        let mut pack = MemoryContextPack::default();

        match node.node_type {
            NodeType::Requirement => pack.requirement = Some(node.clone()),
            NodeType::Decision => pack.decisions.push(node.clone()),
            NodeType::Architecture => pack.architecture.push(node.clone()),
            NodeType::File | NodeType::Function => pack.code.push(node.clone()),
            _ => {} // Other types can be handled as needed
        }

        // Gather upstream and downstream
        // For simplicity, we just look at adjacent nodes. A true implementation would recursively traverse.
        let incoming = self.retrieval_engine.get_neighborhood(
            focal_node_id,
            EdgeDirection::Incoming,
            &[
                EdgeType::Drives,
                EdgeType::Contains,
                EdgeType::Implements,
                EdgeType::DependsOn,
            ],
        )?;

        let outgoing = self.retrieval_engine.get_neighborhood(
            focal_node_id,
            EdgeDirection::Outgoing,
            &[
                EdgeType::Drives,
                EdgeType::Contains,
                EdgeType::Implements,
                EdgeType::DependsOn,
            ],
        )?;

        for neighbor in incoming.into_iter().chain(outgoing) {
            match neighbor.node_type {
                NodeType::Requirement => {
                    if pack.requirement.is_none() {
                        pack.requirement = Some(neighbor);
                    }
                }
                NodeType::Decision => pack.decisions.push(neighbor),
                NodeType::Architecture => pack.architecture.push(neighbor),
                NodeType::File | NodeType::Function => pack.code.push(neighbor),
                _ => {}
            }
        }

        // Extract Governance
        if let Some(owners) = node.properties.get("owners") {
            if let Some(arr) = owners.as_array() {
                if let Some(first) = arr.first() {
                    pack.governance.explicit_owner = Some(first.as_str().unwrap_or("").to_string());
                }
            }
        }
        if let Some(approvers) = node.properties.get("approvers") {
            if let Some(arr) = approvers.as_array() {
                pack.governance.approvers = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
            }
        }

        // Extract Completeness
        if let Some(missing) = node.properties.get("missing_links") {
            if let Some(arr) = missing.as_array() {
                pack.completeness.missing_links = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
            }
        }
        pack.completeness.orphan_status = node
            .properties
            .get("orphan")
            .is_some_and(|v| v.as_bool().unwrap_or(false));

        // Extract Drift
        if node.properties.get("has_drift").is_some_and(|v| v.as_bool().unwrap_or(false)) {
            pack.drift.push(node.clone());
        }

        // Extract Staleness
        // E.g. staleness based on a timestamp diff
        let now = ares_core::types::event::now_micros();
        let age_micros = now.saturating_sub(node.updated_at);
        let age_days = (age_micros / (1_000_000 * 60 * 60 * 24)) as f32;
        if age_days > 180.0 {
            pack.staleness.push(node);
        }

        Ok(pack)
    }
}
