use crate::dag::models::PlanDag;
use ares_core::AresError;
use std::collections::{HashMap, HashSet};

pub struct DagValidator;

impl DagValidator {
    /// Enforces Rule 6: Cycle detection and dependency validation.
    pub fn validate(dag: &PlanDag) -> Result<(), AresError> {
        Self::validate_dependencies(dag)?;
        Self::validate_connected(dag)?;
        Self::detect_cycles(dag)?;
        Ok(())
    }

    fn validate_dependencies(dag: &PlanDag) -> Result<(), AresError> {
        let node_ids: HashSet<&str> = dag.nodes.iter().map(|n| n.id.as_str()).collect();

        for edge in &dag.edges {
            if !node_ids.contains(edge.source.as_str()) {
                return Err(AresError::validation(format!(
                    "Edge references unknown source node: {}",
                    edge.source
                )));
            }
            if !node_ids.contains(edge.target.as_str()) {
                return Err(AresError::validation(format!(
                    "Edge references unknown target node: {}",
                    edge.target
                )));
            }
        }

        Ok(())
    }

    fn detect_cycles(dag: &PlanDag) -> Result<(), AresError> {
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &dag.edges {
            adj.entry(&edge.source).or_default().push(&edge.target);
        }

        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in &dag.nodes {
            if Self::is_cyclic(node.id.as_str(), &adj, &mut visited, &mut rec_stack) {
                return Err(AresError::validation(
                    "Cycle detected in Plan DAG. A plan must be a Directed Acyclic Graph.",
                ));
            }
        }

        Ok(())
    }

    fn is_cyclic<'a>(
        node: &'a str,
        adj: &HashMap<&str, Vec<&'a str>>,
        visited: &mut HashSet<&'a str>,
        rec_stack: &mut HashSet<&'a str>,
    ) -> bool {
        if !visited.contains(node) {
            visited.insert(node);
            rec_stack.insert(node);

            if let Some(neighbors) = adj.get(node) {
                for &neighbor in neighbors {
                    if (!visited.contains(neighbor)
                        && Self::is_cyclic(neighbor, adj, visited, rec_stack))
                        || rec_stack.contains(neighbor)
                    {
                        return true;
                    }
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    fn validate_connected(dag: &PlanDag) -> Result<(), AresError> {
        if dag.nodes.is_empty() {
            return Ok(());
        }

        // Build undirected adjacency list to check for weakly connected component
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &dag.edges {
            adj.entry(&edge.source).or_default().push(&edge.target);
            adj.entry(&edge.target).or_default().push(&edge.source);
        }

        let mut visited = HashSet::new();
        let mut queue = vec![dag.nodes[0].id.as_str()];
        visited.insert(dag.nodes[0].id.as_str());

        while let Some(node) = queue.pop() {
            if let Some(neighbors) = adj.get(node) {
                for &neighbor in neighbors {
                    if visited.insert(neighbor) {
                        queue.push(neighbor);
                    }
                }
            }
        }

        if visited.len() != dag.nodes.len() {
            return Err(AresError::validation(
                "Disconnected Plan DAG detected. All nodes must be part of a single workflow.",
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::models::{DagEdge, DagNode};

    fn make_node(id: &str) -> DagNode {
        DagNode {
            id: id.to_string(),
            title: "".to_string(),
            estimated_duration: 1.0,
            cost: 0.0,
        }
    }

    fn make_edge(source: &str, target: &str) -> DagEdge {
        DagEdge {
            source: source.to_string(),
            target: target.to_string(),
        }
    }

    #[test]
    fn test_valid_sequential_dag() {
        let dag = PlanDag {
            nodes: vec![make_node("A"), make_node("B"), make_node("C")],
            edges: vec![make_edge("A", "B"), make_edge("B", "C")],
        };
        assert!(DagValidator::validate(&dag).is_ok());
    }

    #[test]
    fn test_diamond_dag() {
        let dag = PlanDag {
            nodes: vec![
                make_node("A"),
                make_node("B"),
                make_node("C"),
                make_node("D"),
            ],
            edges: vec![
                make_edge("A", "B"),
                make_edge("A", "C"),
                make_edge("B", "D"),
                make_edge("C", "D"),
            ],
        };
        assert!(DagValidator::validate(&dag).is_ok());
    }

    #[test]
    fn test_cycle_detection() {
        let dag = PlanDag {
            nodes: vec![make_node("A"), make_node("B"), make_node("C")],
            edges: vec![
                make_edge("A", "B"),
                make_edge("B", "C"),
                make_edge("C", "A"),
            ],
        };
        let err = DagValidator::validate(&dag);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Cycle detected"));
    }

    #[test]
    fn test_self_loop() {
        let dag = PlanDag {
            nodes: vec![make_node("A")],
            edges: vec![make_edge("A", "A")],
        };
        let err = DagValidator::validate(&dag);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Cycle detected"));
    }

    #[test]
    fn test_disconnected_dag() {
        let dag = PlanDag {
            nodes: vec![
                make_node("A"),
                make_node("B"),
                make_node("X"),
                make_node("Y"),
            ],
            edges: vec![make_edge("A", "B"), make_edge("X", "Y")],
        };
        let err = DagValidator::validate(&dag);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Disconnected"));
    }

    #[test]
    fn test_large_dag() {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        for i in 0..100 {
            nodes.push(make_node(&format!("N{}", i)));
        }
        for i in 0..99 {
            edges.push(make_edge(&format!("N{}", i), &format!("N{}", i + 1)));
            if i + 2 < 100 {
                edges.push(make_edge(&format!("N{}", i), &format!("N{}", i + 2)));
            }
        }
        let dag = PlanDag { nodes, edges };
        assert!(DagValidator::validate(&dag).is_ok());
    }
}
