#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::format_in_format_args)]
use crate::models::MemoryGap;
use ares_core::types::node::NodeType;
use ares_core::{AresError, ProjectId};
use ares_store::{SqliteGraphRepository, Store};

/// GapEngine detects structural breaks in the memory hierarchy.
/// It finds nodes that should have upstream parents but don't,
/// revealing where ARES' memory is incomplete.
///
/// This is strategically important: ARES finds what is MISSING,
/// which is a stronger moat than simple reasoning.
pub struct GapEngine {
    store: Store,
}

impl GapEngine {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    /// Scans the graph for hierarchy gaps within a project.
    /// For each node type that requires an upstream parent,
    /// checks if that parent edge exists.
    pub fn detect_gaps(&self, project_id: &ProjectId) -> Result<Vec<MemoryGap>, AresError> {
        let repo = SqliteGraphRepository::new(self.store.clone());
        let mut gaps = Vec::new();

        let all_nodes = repo.get_all_nodes(project_id)?;

        for node in &all_nodes {
            let expected = match node.node_type {
                NodeType::Decision => Some("Requirement"),
                NodeType::Architecture => Some("Decision"),
                NodeType::File | NodeType::Folder => Some("Architecture"),
                NodeType::Test => Some("Code"),
                NodeType::RuntimeSignal => Some("Test"),
                NodeType::Outcome => Some("RuntimeSignal"),
                _ => None,
            };

            if let Some(expected_type_name) = expected {
                // Check if this node has any valid upstream edges
                let incoming_edges = repo.get_edges_to(&node.id)?;
                let has_valid_parent = incoming_edges.iter().any(|edge| {
                    if let Ok(Some(parent)) = repo.get_node(&edge.from_node_id) {
                        match (&node.node_type, &parent.node_type) {
                            (NodeType::Decision, NodeType::Requirement) => true,
                            (NodeType::Architecture, NodeType::Decision) => true,
                            (NodeType::File, NodeType::Architecture) => true,
                            (NodeType::Folder, NodeType::Architecture) => true,
                            (
                                NodeType::Test,
                                NodeType::File
                                | NodeType::Function
                                | NodeType::Method
                                | NodeType::Class
                                | NodeType::Struct
                                | NodeType::Enum
                                | NodeType::Trait
                                | NodeType::Module,
                            ) => true,
                            (NodeType::RuntimeSignal, NodeType::Test) => true,
                            (NodeType::Outcome, NodeType::RuntimeSignal) => true,
                            _ => false,
                        }
                    } else {
                        false
                    }
                });

                if !has_valid_parent {
                    gaps.push(MemoryGap {
                        from_type: expected_type_name.to_string(),
                        to_type: format!("{:?}", node.node_type),
                        node_id: node.id.to_string(),
                        gap_description: format!(
                            "{} '{}' has no upstream {} parent",
                            format!("{:?}", node.node_type),
                            node.label,
                            expected_type_name
                        ),
                        confidence: 1.0,
                    });
                }
            }
        }

        // Deterministic sorting
        gaps.sort_by(|a, b| a.node_id.cmp(&b.node_id));

        Ok(gaps)
    }
}
