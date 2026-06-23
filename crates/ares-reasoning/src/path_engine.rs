use crate::models::{MissingMemory, TraceResult, TraceStatus};
use ares_core::types::node::{GraphNode, NodeType};
use ares_core::AresError;
use ares_store::{SqliteGraphRepository, Store};
use std::collections::{HashSet, VecDeque};

pub struct PathEngine {
    store: Store,
}

impl PathEngine {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    /// Recursively trace upstream dependencies with full tracking.
    /// Returns TraceResult with nodes, status, missing memory, path, and distances.
    /// NEVER panics on incomplete graphs — gaps are first-class information.
    pub fn trace_upstream(&self, start_id: &str) -> Result<TraceResult, AresError> {
        let mut visited = HashSet::new();
        let mut queue: VecDeque<(GraphNode, usize)> = VecDeque::new(); // (node, distance)
        let mut results = Vec::new();
        let mut path = Vec::new();
        let mut distances = Vec::new();
        let mut missing = Vec::new();

        let mut edges_visited = 0;
        let mut query_count = 0;

        let repo = SqliteGraphRepository::new(self.store.clone());

        let start_node_opt = repo.get_node(&start_id.into())?;
        let start_node = match start_node_opt {
            Some(node) => node,
            None => {
                return Ok(TraceResult {
                    nodes: vec![],
                    status: TraceStatus::Orphaned,
                    missing: vec![MissingMemory {
                        node_id: start_id.to_string(),
                        expected_type: "any".to_string(),
                        actual_parent: None,
                    }],
                    path: vec![],
                    distances: vec![],
                    nodes_visited: 0,
                    edges_visited: 0,
                    max_depth: 0,
                    query_count: 1,
                });
            }
        };

        path.push(start_node.label.clone());
        queue.push_back((start_node.clone(), 0));

        // Track which hierarchy levels we discovered
        let mut found_types: HashSet<String> = HashSet::new();
        found_types.insert(format!("{:?}", start_node.node_type));

        while let Some((current, dist)) = queue.pop_front() {
            if visited.contains(&current.id) {
                continue;
            }
            visited.insert(current.id.clone());

            query_count += 1;
            let outgoing_edges = repo.get_edges_from(&current.id)?;
            edges_visited += outgoing_edges.len();
            let mut found_upstream = false;

            for edge in outgoing_edges {
                query_count += 1;
                if let Some(upstream_node) = repo.get_node(&edge.to_node_id)? {
                    if self.is_valid_upstream(&current.node_type, &upstream_node.node_type) {
                        let new_dist = dist + 1;
                        found_upstream = true;
                        found_types.insert(format!("{:?}", upstream_node.node_type));
                        path.push(upstream_node.label.clone());
                        distances.push((upstream_node.label.clone(), new_dist));
                        queue.push_back((upstream_node.clone(), new_dist));
                        results.push(upstream_node);
                    }
                }
            }

            // Gap detection: if this node should have an upstream parent but doesn't
            if !found_upstream && dist > 0 {
                if let Some(expected) = self.expected_upstream_type(&current.node_type) {
                    missing.push(MissingMemory {
                        node_id: current.id.to_string(),
                        expected_type: expected.to_string(),
                        actual_parent: None,
                    });
                }
            }
        }

        // Deterministic sorting — prevents HashMap/HashSet ordering variance
        results.sort_by(|a, b| a.id.cmp(&b.id));
        distances.sort_by(|a, b| a.0.cmp(&b.0));

        // Determine trace status
        let status = if results.is_empty() {
            TraceStatus::Orphaned
        } else if missing.is_empty() && self.is_chain_complete(&found_types, &start_node.node_type)
        {
            TraceStatus::Complete
        } else if !missing.is_empty() {
            TraceStatus::GapDetected
        } else {
            TraceStatus::Partial
        };

        Ok(TraceResult {
            nodes: results,
            status,
            missing,
            path,
            distances: distances.clone(),
            nodes_visited: visited.len(),
            edges_visited,
            max_depth: distances.iter().map(|(_, d)| *d).max().unwrap_or(0),
            query_count,
        })
    }

    /// Recursively trace downstream consequences with distance tracking.
    pub fn trace_downstream(&self, start_id: &str) -> Result<TraceResult, AresError> {
        let mut visited = HashSet::new();
        let mut queue: VecDeque<(GraphNode, usize)> = VecDeque::new();
        let mut results = Vec::new();
        let mut path = Vec::new();
        let mut distances = Vec::new();
        let missing = Vec::new();

        let mut edges_visited = 0;
        let mut query_count = 0;

        let repo = SqliteGraphRepository::new(self.store.clone());

        let start_node_opt = repo.get_node(&start_id.into())?;
        let start_node = match start_node_opt {
            Some(node) => node,
            None => {
                return Ok(TraceResult {
                    nodes: vec![],
                    status: TraceStatus::Orphaned,
                    missing: vec![MissingMemory {
                        node_id: start_id.to_string(),
                        expected_type: "any".to_string(),
                        actual_parent: None,
                    }],
                    path: vec![],
                    distances: vec![],
                    nodes_visited: 0,
                    edges_visited: 0,
                    max_depth: 0,
                    query_count: 1,
                });
            }
        };

        path.push(start_node.label.clone());
        queue.push_back((start_node, 0));

        while let Some((current, dist)) = queue.pop_front() {
            if visited.contains(&current.id) {
                continue;
            }
            visited.insert(current.id.clone());

            query_count += 1;
            let incoming_edges = repo.get_edges_to(&current.id)?;
            edges_visited += incoming_edges.len();

            for edge in incoming_edges {
                query_count += 1;
                if let Some(downstream_node) = repo.get_node(&edge.from_node_id)? {
                    if self.is_valid_downstream(&current.node_type, &downstream_node.node_type) {
                        let new_dist = dist + 1;
                        path.push(downstream_node.label.clone());
                        distances.push((downstream_node.label.clone(), new_dist));
                        queue.push_back((downstream_node.clone(), new_dist));
                        results.push(downstream_node);
                    }
                }
            }
        }

        // Deterministic sorting
        results.sort_by(|a, b| a.id.cmp(&b.id));
        distances.sort_by(|a, b| a.0.cmp(&b.0));

        let status = if results.is_empty() {
            TraceStatus::Orphaned
        } else {
            TraceStatus::Complete
        };

        Ok(TraceResult {
            nodes: results,
            status,
            missing,
            path,
            distances: distances.clone(),
            nodes_visited: visited.len(),
            edges_visited,
            max_depth: distances.iter().map(|(_, d)| *d).max().unwrap_or(0),
            query_count,
        })
    }

    /// Enforces upstream hierarchy: Code -> Architecture -> Decision -> Requirement
    fn is_valid_upstream(&self, downstream: &NodeType, upstream: &NodeType) -> bool {
        match (downstream, upstream) {
            (NodeType::Outcome, NodeType::RuntimeSignal) => true,
            (NodeType::RuntimeSignal, NodeType::Test) => true,
            (NodeType::Test, NodeType::File) => true,
            (NodeType::Test, NodeType::Folder) => true,
            (NodeType::File, NodeType::Architecture) => true,
            (NodeType::Folder, NodeType::Architecture) => true,
            (NodeType::Architecture, NodeType::Decision) => true,
            (NodeType::Decision, NodeType::Requirement) => true,
            // Cross-hierarchy hops are blocked
            _ => false,
        }
    }

    /// Enforces downstream hierarchy: Requirement -> Decision -> Architecture -> Code
    fn is_valid_downstream(&self, upstream: &NodeType, downstream: &NodeType) -> bool {
        // Downstream is just the inverse of upstream
        self.is_valid_upstream(downstream, upstream)
    }

    /// What upstream type is expected for a given node type?
    fn expected_upstream_type(&self, node_type: &NodeType) -> Option<&'static str> {
        match node_type {
            NodeType::Decision => Some("Requirement"),
            NodeType::Architecture => Some("Decision"),
            NodeType::File | NodeType::Folder => Some("Architecture"),
            NodeType::Test => Some("File/Folder"),
            NodeType::RuntimeSignal => Some("Test"),
            NodeType::Outcome => Some("RuntimeSignal"),
            _ => None,
        }
    }

    /// Check if the traversal found all expected hierarchy levels above the start node
    fn is_chain_complete(&self, found_types: &HashSet<String>, start_type: &NodeType) -> bool {
        // For a File node, complete chain = Architecture + Decision + Requirement
        let required = match start_type {
            NodeType::File | NodeType::Folder => vec!["Architecture", "Decision", "Requirement"],
            NodeType::Test => vec!["File", "Architecture", "Decision", "Requirement"],
            NodeType::Architecture => vec!["Decision", "Requirement"],
            NodeType::Decision => vec!["Requirement"],
            _ => return true,
        };

        for r in required {
            if !found_types.contains(r) {
                return false;
            }
        }
        true
    }
}
