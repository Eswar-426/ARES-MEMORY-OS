use crate::store::KnowledgeGraphStore;
use ares_core::AresError;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct IntegrityReport {
    pub dangling_nodes: Vec<String>,
    pub orphan_edges: Vec<String>,
    pub invalid_references: Vec<String>,
    pub invalid_directions: Vec<String>,
    pub cycles: Vec<Vec<String>>,
}

pub struct GraphIntegrityValidator {
    store: Arc<KnowledgeGraphStore>,
}

impl GraphIntegrityValidator {
    pub fn new(store: Arc<KnowledgeGraphStore>) -> Self {
        Self { store }
    }

    fn is_valid_edge(source: &crate::models::NodeType, edge: &crate::models::EdgeType, target: &crate::models::NodeType) -> bool {
        use crate::models::{NodeType, EdgeType};
        match edge {
            EdgeType::ResultsIn => matches!((source, target), 
                (NodeType::Requirement, NodeType::Decision) | 
                (NodeType::Decision, NodeType::Architecture)
            ),
            EdgeType::ImplementedBy => matches!((source, target),
                (NodeType::Requirement, NodeType::CodeArtifact)
            ),
            EdgeType::Implements => matches!((source, target),
                (NodeType::Architecture, NodeType::CodeArtifact)
            ),
            EdgeType::ValidatedBy => matches!((source, target),
                (NodeType::CodeArtifact, NodeType::Test) |
                (NodeType::Architecture, NodeType::Test) |
                (NodeType::Requirement, NodeType::Test)
            ),
            EdgeType::Exhibits => matches!((source, target),
                (NodeType::Test, NodeType::RuntimeSignal)
            ),
            EdgeType::Causes => matches!((source, target),
                (NodeType::RuntimeSignal, NodeType::Outcome) |
                (NodeType::Gap, NodeType::RootCause)
            ),
            EdgeType::Resolves => matches!((source, target),
                (NodeType::Resolution, NodeType::Gap)
            ),
            EdgeType::OwnedBy | EdgeType::ApprovedBy => matches!(target, NodeType::Owner),
            EdgeType::Supersedes => source == target,
            EdgeType::DerivedFrom => true,
            EdgeType::TracesTo => true,
            EdgeType::References => true,
            EdgeType::Drives => matches!((source, target),
                (NodeType::Decision, NodeType::CodeArtifact)
            ),
            EdgeType::Supports => matches!((source, target),
                (NodeType::Evidence, NodeType::Decision)
            ),
            EdgeType::DependsOn | EdgeType::SupportedBy => true,
            EdgeType::Contains => matches!((source, target),
                (NodeType::Repository, NodeType::CodeArtifact) |
                (NodeType::CodeArtifact, NodeType::CodeArtifact)
            ),
            EdgeType::OccurredIn => matches!((source, target),
                (NodeType::RepositorySnapshot, NodeType::RepositoryEvent) |
                (NodeType::RepositoryEvent, _)
            ),
            EdgeType::GeneratedFrom => matches!((source, target),
                (NodeType::RepositoryEvent, _)
            ),
            EdgeType::HasGap => matches!((source, target),
                (NodeType::RepositoryEvent, NodeType::KnowledgeGap)
            ),
        }
    }

    pub fn validate_graph(&self) -> Result<IntegrityReport, AresError> {
        let mut report = IntegrityReport::default();
        let raw_store = self.store.get_raw_store();
        let conn = raw_store.get_conn()?;

        // 1. Find Orphan Edges and Direction Violations
        let mut stmt = conn.prepare(
            "SELECT id, source_entity, target_entity, relationship_type FROM graph_relationships"
        ).map_err(|e| AresError::Database(e.to_string()))?;
        
        let mut edges = Vec::new();
        let mut edge_rows = stmt.query([]).map_err(|e| AresError::Database(e.to_string()))?;
        
        while let Some(row) = edge_rows.next().map_err(|e| AresError::Database(e.to_string()))? {
            let id: String = row.get(0).map_err(|e| AresError::Database(e.to_string()))?;
            let source: String = row.get(1).map_err(|e| AresError::Database(e.to_string()))?;
            let target: String = row.get(2).map_err(|e| AresError::Database(e.to_string()))?;
            let edge_type_str: String = row.get(3).map_err(|e| AresError::Database(e.to_string()))?;
            
            let edge_type = serde_json::from_value(serde_json::json!(edge_type_str))
                .unwrap_or(crate::models::EdgeType::References);
            
            edges.push((id, source, target, edge_type));
        }

        let mut stmt = conn.prepare("SELECT id, entity_type FROM graph_entities")
            .map_err(|e| AresError::Database(e.to_string()))?;
        let mut node_rows = stmt.query([]).map_err(|e| AresError::Database(e.to_string()))?;
        let mut valid_nodes = HashMap::new();
        while let Some(row) = node_rows.next().map_err(|e| AresError::Database(e.to_string()))? {
            let id: String = row.get(0).map_err(|e| AresError::Database(e.to_string()))?;
            let node_type_str: String = row.get(1).map_err(|e| AresError::Database(e.to_string()))?;
            
            let node_type = serde_json::from_value(serde_json::json!(node_type_str))
                .unwrap_or(crate::models::NodeType::CodeArtifact);
                
            valid_nodes.insert(id, node_type);
        }

        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        let mut node_degrees: HashMap<String, usize> = HashMap::new();

        for node in valid_nodes.keys() {
            node_degrees.insert(node.clone(), 0);
        }

        for (edge_id, source, target, edge_type) in &edges {
            let mut is_orphan = false;
            
            let source_type = valid_nodes.get(source);
            let target_type = valid_nodes.get(target);

            if source_type.is_none() {
                report.invalid_references.push(format!("Edge {} references missing source {}", edge_id, source));
                is_orphan = true;
            } else {
                *node_degrees.entry(source.clone()).or_insert(0) += 1;
            }

            if target_type.is_none() {
                report.invalid_references.push(format!("Edge {} references missing target {}", edge_id, target));
                is_orphan = true;
            } else {
                *node_degrees.entry(target.clone()).or_insert(0) += 1;
            }

            if is_orphan {
                report.orphan_edges.push(edge_id.clone());
            } else {
                // Check direction
                if let (Some(s_type), Some(t_type)) = (source_type, target_type) {
                    if !Self::is_valid_edge(s_type, edge_type, t_type) {
                        report.invalid_directions.push(format!(
                            "Invalid edge {}: {:?} -> {:?} -> {:?}",
                            edge_id, s_type, edge_type, t_type
                        ));
                    }
                }
                
                // For cycle detection
                adj_list.entry(source.clone()).or_default().push(target.clone());
            }
        }

        // 2. Find Dangling Nodes
        // Nodes with 0 edges connected
        for (node, degree) in node_degrees {
            if degree == 0 {
                report.dangling_nodes.push(node);
            }
        }

        // 3. Cycle Detection (DFS based on adj_list)
        let keys_set: HashSet<String> = valid_nodes.keys().cloned().collect();
        report.cycles = detect_cycles(&keys_set, &adj_list);

        Ok(report)
    }
}

// Simple DFS to find cycles
fn detect_cycles(
    nodes: &HashSet<String>, 
    adj: &HashMap<String, Vec<String>>
) -> Vec<Vec<String>> {
    let mut cycles = Vec::new();
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    let mut path = Vec::new();

    for node in nodes {
        if !visited.contains(node) {
            dfs_cycle(node, adj, &mut visited, &mut rec_stack, &mut path, &mut cycles);
        }
    }
    cycles
}

fn dfs_cycle(
    u: &String,
    adj: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    rec_stack: &mut HashSet<String>,
    path: &mut Vec<String>,
    cycles: &mut Vec<Vec<String>>
) {
    visited.insert(u.clone());
    rec_stack.insert(u.clone());
    path.push(u.clone());

    if let Some(neighbors) = adj.get(u) {
        for v in neighbors {
            if !visited.contains(v) {
                dfs_cycle(v, adj, visited, rec_stack, path, cycles);
            } else if rec_stack.contains(v) {
                // Cycle found
                if let Some(pos) = path.iter().position(|n| n == v) {
                    let mut cycle = path[pos..].to_vec();
                    cycle.push(v.clone());
                    cycles.push(cycle);
                }
            }
        }
    }

    rec_stack.remove(u);
    path.pop();
}
