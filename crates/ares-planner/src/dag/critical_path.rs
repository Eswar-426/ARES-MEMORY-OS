use crate::dag::models::{DagNode, PlanDag};
use std::collections::HashMap;

pub struct CriticalPathAnalysis {
    pub max_duration: f64,
    pub critical_nodes: Vec<String>,
    pub parallelism_factor: f64,
}

impl CriticalPathAnalysis {
    /// Calculates the longest path (critical path) and analyzes possible parallelism.
    pub fn calculate(dag: &PlanDag) -> Self {
        // If empty DAG, return 0
        if dag.nodes.is_empty() {
            return Self {
                max_duration: 0.0,
                critical_nodes: vec![],
                parallelism_factor: 1.0,
            };
        }

        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut node_map: HashMap<&str, &DagNode> = HashMap::new();

        for node in &dag.nodes {
            node_map.insert(&node.id, node);
            in_degree.insert(&node.id, 0);
        }

        for edge in &dag.edges {
            adj.entry(&edge.source).or_default().push(&edge.target);
            *in_degree.entry(&edge.target).or_insert(0) += 1;
        }

        // Topological sort + DP for longest path
        let mut dp: HashMap<&str, f64> = HashMap::new();
        let mut prev: HashMap<&str, &str> = HashMap::new();
        let mut queue = Vec::new();

        let mut total_duration = 0.0;

        for (id, &degree) in &in_degree {
            if degree == 0 {
                queue.push(*id);
                let duration = node_map[id].estimated_duration;
                dp.insert(id, duration);
                total_duration += duration;
            }
        }

        let mut max_end_node = queue.first().copied().unwrap_or("");
        let mut max_overall_duration = 0.0;

        while let Some(u) = queue.pop() {
            let u_dur = dp[&u];
            if u_dur > max_overall_duration {
                max_overall_duration = u_dur;
                max_end_node = u;
            }

            if let Some(neighbors) = adj.get(u) {
                for &v in neighbors {
                    let v_dur = node_map[v].estimated_duration;
                    if dp.get(v).copied().unwrap_or(0.0) < u_dur + v_dur {
                        dp.insert(v, u_dur + v_dur);
                        prev.insert(v, u);
                    }

                    *in_degree.get_mut(v).unwrap() -= 1;
                    if in_degree[v] == 0 {
                        queue.push(v);
                        total_duration += v_dur;
                    }
                }
            }
        }

        // Reconstruct path
        let mut critical_nodes = Vec::new();
        let mut curr = max_end_node;
        while !curr.is_empty() {
            critical_nodes.push(curr.to_string());
            if let Some(&p) = prev.get(curr) {
                curr = p;
            } else {
                break;
            }
        }
        critical_nodes.reverse();

        // Calculate parallelism factor = Total sequential duration / Critical path duration
        let parallelism_factor = if max_overall_duration > 0.0 {
            total_duration / max_overall_duration
        } else {
            1.0
        };

        Self {
            max_duration: max_overall_duration,
            critical_nodes,
            parallelism_factor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::models::{DagEdge, DagNode};

    fn make_node(id: &str, duration: f64) -> DagNode {
        DagNode {
            id: id.to_string(),
            title: "".to_string(),
            estimated_duration: duration,
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
    fn test_critical_path_sequential() {
        let dag = PlanDag {
            nodes: vec![
                make_node("A", 10.0),
                make_node("B", 20.0),
                make_node("C", 30.0),
            ],
            edges: vec![make_edge("A", "B"), make_edge("B", "C")],
        };
        let analysis = CriticalPathAnalysis::calculate(&dag);
        assert_eq!(analysis.max_duration, 60.0);
        assert_eq!(analysis.critical_nodes, vec!["A", "B", "C"]);
        assert_eq!(analysis.parallelism_factor, 1.0); // 60/60
    }

    #[test]
    fn test_critical_path_diamond() {
        let dag = PlanDag {
            nodes: vec![
                make_node("A", 10.0),
                make_node("B", 50.0), // Long path
                make_node("C", 20.0), // Short path
                make_node("D", 10.0),
            ],
            edges: vec![
                make_edge("A", "B"),
                make_edge("A", "C"),
                make_edge("B", "D"),
                make_edge("C", "D"),
            ],
        };
        let analysis = CriticalPathAnalysis::calculate(&dag);
        assert_eq!(analysis.max_duration, 70.0); // A(10) + B(50) + D(10)
        assert_eq!(analysis.critical_nodes, vec!["A", "B", "D"]);

        let total_dur = 10.0 + 50.0 + 20.0 + 10.0; // 90.0
        assert_eq!(analysis.parallelism_factor, total_dur / 70.0); // 1.2857
    }

    #[test]
    fn test_critical_path_empty() {
        let dag = PlanDag {
            nodes: vec![],
            edges: vec![],
        };
        let analysis = CriticalPathAnalysis::calculate(&dag);
        assert_eq!(analysis.max_duration, 0.0);
        assert!(analysis.critical_nodes.is_empty());
        assert_eq!(analysis.parallelism_factor, 1.0);
    }
}
