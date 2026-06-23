use crate::models::{HierarchySegment, TopologyState};
use ares_core::types::node::NodeType;
use ares_core::{AresError, ProjectId};
use ares_store::{SqliteGraphRepository, Store};
use std::collections::HashSet;

pub struct TopologyEngine {
    store: Store,
}

impl TopologyEngine {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    /// Evaluates the topology state of all nodes in a project.
    pub fn evaluate_topology(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<HierarchySegment>, AresError> {
        let repo = SqliteGraphRepository::new(self.store.clone());
        let all_nodes = repo.get_all_nodes(project_id)?;

        let mut segments = Vec::new();

        for node in &all_nodes {
            let upstream = repo.get_edges_to(&node.id)?;
            let downstream = repo.get_edges_from(&node.id)?;

            if upstream.is_empty() && downstream.is_empty() {
                segments.push(HierarchySegment {
                    node_id: node.id.to_string(),
                    node_type: format!("{:?}", node.node_type),
                    state: TopologyState::Orphaned,
                    missing_downstream: vec![],
                });
                continue;
            }

            // Check if disconnected from the root (Requirement)
            let is_root = matches!(node.node_type, NodeType::Requirement);
            let has_upstream = !upstream.is_empty();

            if !is_root && !has_upstream {
                // It's not a root and has no upstream -> Disconnected
                // (or could be a root of a sub-tree, but effectively disconnected from the main hierarchy)
                // Let's classify as Disconnected for now, we will refine missing downstream later.
            }

            // For down-stream completeness:
            // The ideal chain is: Requirement -> Decision -> Architecture -> (File/Function/Method/Class/Struct/Enum/Trait/Module) -> Test -> RuntimeSignal -> Outcome
            // Let's perform a BFS/DFS downstream to see what types are reachable.
            let reachable_types = self.get_downstream_types(&repo, &node.id)?;

            let missing = Self::calculate_missing_downstream(&node.node_type, &reachable_types);

            let state = if !is_root && !has_upstream {
                TopologyState::Disconnected
            } else if missing.is_empty() {
                TopologyState::Complete
            } else {
                TopologyState::Partial
            };

            segments.push(HierarchySegment {
                node_id: node.id.to_string(),
                node_type: format!("{:?}", node.node_type),
                state,
                missing_downstream: missing,
            });
        }

        Ok(segments)
    }

    fn get_downstream_types(
        &self,
        repo: &SqliteGraphRepository,
        start_node_id: &ares_core::NodeId,
    ) -> Result<Vec<NodeType>, AresError> {
        let mut visited = HashSet::new();
        let mut queue = vec![start_node_id.clone()];
        let mut found_types = Vec::new();

        while let Some(current_id) = queue.pop() {
            if visited.contains(&current_id) {
                continue;
            }
            visited.insert(current_id.clone());

            let downstream_edges = repo.get_edges_from(&current_id)?;
            for edge in downstream_edges {
                if let Ok(Some(child)) = repo.get_node(&edge.to_node_id) {
                    if !found_types.contains(&child.node_type) {
                        found_types.push(child.node_type.clone());
                    }
                    queue.push(child.id.clone());
                }
            }
        }

        Ok(found_types)
    }

    fn calculate_missing_downstream(
        current_type: &NodeType,
        reachable_types: &[NodeType],
    ) -> Vec<String> {
        let mut missing = Vec::new();

        let sequence = vec![
            (vec![NodeType::Requirement], "Decision"),
            (vec![NodeType::Decision], "Architecture"),
            (vec![NodeType::Architecture], "Code"),
            (
                vec![
                    NodeType::File,
                    NodeType::Function,
                    NodeType::Method,
                    NodeType::Class,
                    NodeType::Struct,
                    NodeType::Enum,
                    NodeType::Trait,
                    NodeType::Module,
                ],
                "Test",
            ),
            (vec![NodeType::Test], "RuntimeSignal"),
            (vec![NodeType::RuntimeSignal], "Outcome"),
        ];

        let mut start_checking = false;

        for (types, next_name) in sequence {
            if !start_checking && types.contains(current_type) {
                start_checking = true;
            }
            // If we are evaluating something further down the chain, we don't look back

            if start_checking {
                // Check if reachable_types contains the NEXT expected type
                let found_next = match next_name {
                    "Decision" => reachable_types.contains(&NodeType::Decision),
                    "Architecture" => reachable_types.contains(&NodeType::Architecture),
                    "Code" => reachable_types.iter().any(|t| {
                        matches!(
                            t,
                            NodeType::File
                                | NodeType::Function
                                | NodeType::Method
                                | NodeType::Class
                                | NodeType::Struct
                                | NodeType::Enum
                                | NodeType::Trait
                                | NodeType::Module
                        )
                    }),
                    "Test" => reachable_types.contains(&NodeType::Test),
                    "RuntimeSignal" => reachable_types.contains(&NodeType::RuntimeSignal),
                    "Outcome" => reachable_types.contains(&NodeType::Outcome),
                    _ => false,
                };

                if !found_next {
                    missing.push(next_name.to_string());
                }
            }
        }

        missing
    }
}
