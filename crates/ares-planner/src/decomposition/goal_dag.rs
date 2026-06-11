use crate::models::goal::GoalPriority;
use ares_core::id::GoalId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// The type of relationship between two goal nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalEdgeType {
    /// Target depends on source completing first.
    DependsOn,
    /// Target is a milestone checkpoint.
    Milestone,
    /// Target is a sub-goal of source.
    SubGoal,
}

/// A directed edge in the Goal DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalEdge {
    pub from: GoalId,
    pub to: GoalId,
    pub edge_type: GoalEdgeType,
}

/// A single node in the Goal DAG representing a decomposed sub-goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalNode {
    pub id: GoalId,
    pub title: String,
    pub description: Option<String>,
    pub dependencies: Vec<GoalId>,
    pub priority: GoalPriority,
    pub estimated_cost: f64,
    pub estimated_duration_secs: f64,
    pub depth: u32,
}

/// A directed acyclic graph of decomposed goals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalDag {
    pub nodes: HashMap<GoalId, GoalNode>,
    pub edges: Vec<GoalEdge>,
    pub root_id: GoalId,
}

impl GoalDag {
    pub fn new(root_id: GoalId) -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            root_id,
        }
    }

    pub fn add_node(&mut self, node: GoalNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn add_edge(&mut self, edge: GoalEdge) {
        self.edges.push(edge);
    }

    /// Returns IDs of nodes with no incoming edges.
    pub fn get_roots(&self) -> Vec<GoalId> {
        let has_incoming: HashSet<&GoalId> = self.edges.iter().map(|e| &e.to).collect();
        self.nodes
            .keys()
            .filter(|id| !has_incoming.contains(id))
            .cloned()
            .collect()
    }

    /// Returns IDs of direct children (outgoing edges from `parent`).
    pub fn get_children(&self, parent: &GoalId) -> Vec<GoalId> {
        self.edges
            .iter()
            .filter(|e| &e.from == parent)
            .map(|e| e.to.clone())
            .collect()
    }

    /// Returns IDs of nodes with no outgoing edges (leaf nodes).
    pub fn get_leaves(&self) -> Vec<GoalId> {
        let has_outgoing: HashSet<&GoalId> = self.edges.iter().map(|e| &e.from).collect();
        self.nodes
            .keys()
            .filter(|id| !has_outgoing.contains(id))
            .cloned()
            .collect()
    }

    /// Topological sort using Kahn's algorithm.
    /// Returns `None` if the graph contains a cycle.
    pub fn topological_sort(&self) -> Option<Vec<GoalId>> {
        let mut in_degree: HashMap<GoalId, usize> = HashMap::new();
        for id in self.nodes.keys() {
            in_degree.insert(id.clone(), 0);
        }
        for edge in &self.edges {
            *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
        }

        let mut queue: VecDeque<GoalId> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut sorted = Vec::new();
        while let Some(id) = queue.pop_front() {
            sorted.push(id.clone());
            for edge in &self.edges {
                if edge.from == id {
                    if let Some(deg) = in_degree.get_mut(&edge.to) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(edge.to.clone());
                        }
                    }
                }
            }
        }

        if sorted.len() == self.nodes.len() {
            Some(sorted)
        } else {
            None // cycle detected
        }
    }

    /// Validates that the graph is acyclic.
    pub fn validate_acyclic(&self) -> bool {
        self.topological_sort().is_some()
    }

    /// Total number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Total number of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(title: &str, depth: u32) -> GoalNode {
        GoalNode {
            id: GoalId::new(),
            title: title.to_string(),
            description: None,
            dependencies: Vec::new(),
            priority: GoalPriority::Medium,
            estimated_cost: 10.0,
            estimated_duration_secs: 60.0,
            depth,
        }
    }

    #[test]
    fn empty_dag() {
        let dag = GoalDag::new(GoalId::new());
        assert_eq!(dag.node_count(), 0);
        assert_eq!(dag.edge_count(), 0);
        assert!(dag.validate_acyclic());
    }

    #[test]
    fn single_node_dag() {
        let root = GoalId::new();
        let mut dag = GoalDag::new(root.clone());
        let mut node = make_node("Root", 0);
        node.id = root.clone();
        dag.add_node(node);

        assert_eq!(dag.node_count(), 1);
        assert_eq!(dag.get_roots(), vec![root.clone()]);
        assert_eq!(dag.get_leaves(), vec![root]);
    }

    #[test]
    fn linear_chain() {
        let mut dag = GoalDag::new(GoalId::new());
        let a = make_node("A", 0);
        let mut b = make_node("B", 1);
        let mut c = make_node("C", 2);
        let id_a = a.id.clone();
        let id_b = b.id.clone();
        let id_c = c.id.clone();
        dag.root_id = id_a.clone();

        b.dependencies = vec![id_a.clone()];
        c.dependencies = vec![id_b.clone()];

        dag.add_node(a);
        dag.add_node(b);
        dag.add_node(c);
        dag.add_edge(GoalEdge {
            from: id_a.clone(),
            to: id_b.clone(),
            edge_type: GoalEdgeType::SubGoal,
        });
        dag.add_edge(GoalEdge {
            from: id_b.clone(),
            to: id_c.clone(),
            edge_type: GoalEdgeType::DependsOn,
        });

        assert_eq!(dag.get_roots(), vec![id_a.clone()]);
        assert_eq!(dag.get_leaves(), vec![id_c.clone()]);
        assert_eq!(dag.get_children(&id_a), vec![id_b.clone()]);
        assert_eq!(dag.get_children(&id_b), vec![id_c]);
    }

    #[test]
    fn topological_sort_linear() {
        let id_a = GoalId::new();
        let id_b = GoalId::new();

        let mut dag = GoalDag::new(id_a.clone());
        let mut a = make_node("A", 0);
        a.id = id_a.clone();
        let mut b = make_node("B", 1);
        b.id = id_b.clone();

        dag.add_node(a);
        dag.add_node(b);
        dag.add_edge(GoalEdge {
            from: id_a.clone(),
            to: id_b.clone(),
            edge_type: GoalEdgeType::SubGoal,
        });

        let sorted = dag.topological_sort().unwrap();
        let pos_a = sorted.iter().position(|id| id == &id_a).unwrap();
        let pos_b = sorted.iter().position(|id| id == &id_b).unwrap();
        assert!(pos_a < pos_b);
    }

    #[test]
    fn cycle_detected() {
        let id_a = GoalId::new();
        let id_b = GoalId::new();

        let mut dag = GoalDag::new(id_a.clone());
        let mut a = make_node("A", 0);
        a.id = id_a.clone();
        let mut b = make_node("B", 1);
        b.id = id_b.clone();

        dag.add_node(a);
        dag.add_node(b);
        dag.add_edge(GoalEdge {
            from: id_a.clone(),
            to: id_b.clone(),
            edge_type: GoalEdgeType::SubGoal,
        });
        dag.add_edge(GoalEdge {
            from: id_b.clone(),
            to: id_a.clone(),
            edge_type: GoalEdgeType::DependsOn,
        });

        assert!(!dag.validate_acyclic());
        assert!(dag.topological_sort().is_none());
    }

    #[test]
    fn diamond_dag() {
        let id_a = GoalId::new();
        let id_b = GoalId::new();
        let id_c = GoalId::new();
        let id_d = GoalId::new();

        let mut dag = GoalDag::new(id_a.clone());
        for (id, title) in [(&id_a, "A"), (&id_b, "B"), (&id_c, "C"), (&id_d, "D")] {
            let mut n = make_node(title, 0);
            n.id = id.clone();
            dag.add_node(n);
        }

        // A -> B, A -> C, B -> D, C -> D
        dag.add_edge(GoalEdge {
            from: id_a.clone(),
            to: id_b.clone(),
            edge_type: GoalEdgeType::SubGoal,
        });
        dag.add_edge(GoalEdge {
            from: id_a.clone(),
            to: id_c.clone(),
            edge_type: GoalEdgeType::SubGoal,
        });
        dag.add_edge(GoalEdge {
            from: id_b.clone(),
            to: id_d.clone(),
            edge_type: GoalEdgeType::DependsOn,
        });
        dag.add_edge(GoalEdge {
            from: id_c.clone(),
            to: id_d.clone(),
            edge_type: GoalEdgeType::DependsOn,
        });

        assert!(dag.validate_acyclic());
        let sorted = dag.topological_sort().unwrap();
        let pos_a = sorted.iter().position(|id| id == &id_a).unwrap();
        let pos_d = sorted.iter().position(|id| id == &id_d).unwrap();
        assert!(pos_a < pos_d);
    }

    #[test]
    fn get_children_empty() {
        let dag = GoalDag::new(GoalId::new());
        assert!(dag.get_children(&GoalId::new()).is_empty());
    }

    #[test]
    fn edge_types_serialize() {
        let edge = GoalEdge {
            from: GoalId::new(),
            to: GoalId::new(),
            edge_type: GoalEdgeType::Milestone,
        };
        let json = serde_json::to_string(&edge).unwrap();
        assert!(json.contains("Milestone"));
    }

    #[test]
    fn node_serialization() {
        let node = make_node("Test", 2);
        let json = serde_json::to_string(&node).unwrap();
        let back: GoalNode = serde_json::from_str(&json).unwrap();
        assert_eq!(back.title, "Test");
        assert_eq!(back.depth, 2);
    }

    #[test]
    fn dag_serialization() {
        let root_id = GoalId::new();
        let mut dag = GoalDag::new(root_id.clone());
        let mut node = make_node("root", 0);
        node.id = root_id;
        dag.add_node(node);

        let json = serde_json::to_string(&dag).unwrap();
        let back: GoalDag = serde_json::from_str(&json).unwrap();
        assert_eq!(back.node_count(), 1);
    }

    #[test]
    fn multiple_roots() {
        let id_a = GoalId::new();
        let id_b = GoalId::new();
        let id_c = GoalId::new();

        let mut dag = GoalDag::new(id_a.clone());
        for (id, t) in [(&id_a, "A"), (&id_b, "B"), (&id_c, "C")] {
            let mut n = make_node(t, 0);
            n.id = id.clone();
            dag.add_node(n);
        }
        // Only A -> C, B has no edges → two roots
        dag.add_edge(GoalEdge {
            from: id_a.clone(),
            to: id_c.clone(),
            edge_type: GoalEdgeType::SubGoal,
        });

        let roots = dag.get_roots();
        assert_eq!(roots.len(), 2);
        assert!(roots.contains(&id_a));
        assert!(roots.contains(&id_b));
    }

    #[test]
    fn leaves_with_multiple_paths() {
        let id_a = GoalId::new();
        let id_b = GoalId::new();
        let id_c = GoalId::new();

        let mut dag = GoalDag::new(id_a.clone());
        for (id, t) in [(&id_a, "A"), (&id_b, "B"), (&id_c, "C")] {
            let mut n = make_node(t, 0);
            n.id = id.clone();
            dag.add_node(n);
        }
        // A -> B, A -> C → two leaves
        dag.add_edge(GoalEdge {
            from: id_a.clone(),
            to: id_b.clone(),
            edge_type: GoalEdgeType::SubGoal,
        });
        dag.add_edge(GoalEdge {
            from: id_a.clone(),
            to: id_c.clone(),
            edge_type: GoalEdgeType::SubGoal,
        });

        let leaves = dag.get_leaves();
        assert_eq!(leaves.len(), 2);
        assert!(leaves.contains(&id_b));
        assert!(leaves.contains(&id_c));
    }
}
