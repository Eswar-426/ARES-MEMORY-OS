use ares_core::{ArchitectureHealthReport, KnowledgeGraph, NodeId};
use std::collections::{HashMap, HashSet};

pub struct ArchitecturalAnalysisEngine {}

impl ArchitecturalAnalysisEngine {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ArchitecturalAnalysisEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchitecturalAnalysisEngine {
    pub fn analyze(&self, kg: &KnowledgeGraph) -> ArchitectureHealthReport {
        let mut fan_in: HashMap<String, usize> = HashMap::new();
        let mut fan_out: HashMap<String, usize> = HashMap::new();
        let mut node_map = HashMap::new();

        for node in &kg.nodes {
            let id = node.id.as_str().to_string();
            fan_in.insert(id.clone(), 0);
            fan_out.insert(id.clone(), 0);
            node_map.insert(id, node.clone());
        }

        let mut adj = HashMap::new();
        for edge in &kg.edges {
            let from = edge.from_node_id.as_str().to_string();
            let to = edge.to_node_id.as_str().to_string();

            *fan_out.entry(from.clone()).or_insert(0) += 1;
            *fan_in.entry(to.clone()).or_insert(0) += 1;

            adj.entry(from).or_insert_with(Vec::new).push(to);
        }

        let mut fan_in_hotspots = Vec::new();
        let mut fan_out_hotspots = Vec::new();
        let mut unstable_modules = Vec::new();
        let mut orphan_modules = Vec::new();
        let mut dependency_bottlenecks = Vec::new();

        for (id, &in_degree) in &fan_in {
            let out_degree = *fan_out.get(id).unwrap_or(&0);

            if in_degree > 10 {
                fan_in_hotspots.push(NodeId::from(id.as_str()));
            }
            if out_degree > 10 {
                fan_out_hotspots.push(NodeId::from(id.as_str()));
            }

            // Unstable: high fan-out, low fan-in
            if out_degree > 5 && in_degree == 0 {
                unstable_modules.push(NodeId::from(id.as_str()));
            }

            // Orphan: 0 fan-in and 0 fan-out
            if in_degree == 0 && out_degree == 0 {
                orphan_modules.push(NodeId::from(id.as_str()));
            }

            // Bottlenecks: high fan-in AND high fan-out
            if in_degree > 5 && out_degree > 5 {
                dependency_bottlenecks.push(NodeId::from(id.as_str()));
            }
        }

        let cycles = self.detect_cycles(&kg.nodes, &adj);

        ArchitectureHealthReport {
            fan_in_hotspots,
            fan_out_hotspots,
            unstable_modules,
            orphan_modules,
            dependency_bottlenecks,
            cycles,
        }
    }

    fn detect_cycles(
        &self,
        nodes: &[ares_core::GraphNode],
        adj: &HashMap<String, Vec<String>>,
    ) -> Vec<Vec<NodeId>> {
        // Tarjan's strongly connected components algorithm
        let mut index = 0;
        let mut stack = Vec::new();
        let mut on_stack = HashSet::new();
        let mut indices: HashMap<String, usize> = HashMap::new();
        let mut lowlinks: HashMap<String, usize> = HashMap::new();
        let mut sccs = Vec::new();

        for node in nodes {
            let v = node.id.as_str().to_string();
            if !indices.contains_key(&v) {
                self.strongconnect(
                    &v,
                    adj,
                    &mut index,
                    &mut stack,
                    &mut on_stack,
                    &mut indices,
                    &mut lowlinks,
                    &mut sccs,
                );
            }
        }

        // Filter out single-node SCCs without self-loops
        sccs.into_iter()
            .filter(|scc| scc.len() > 1 || adj.get(&scc[0]).is_some_and(|n| n.contains(&scc[0])))
            .map(|scc| scc.into_iter().map(NodeId::from).collect())
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    fn strongconnect(
        &self,
        v: &String,
        adj: &HashMap<String, Vec<String>>,
        index: &mut usize,
        stack: &mut Vec<String>,
        on_stack: &mut HashSet<String>,
        indices: &mut HashMap<String, usize>,
        lowlinks: &mut HashMap<String, usize>,
        sccs: &mut Vec<Vec<String>>,
    ) {
        indices.insert(v.clone(), *index);
        lowlinks.insert(v.clone(), *index);
        *index += 1;
        stack.push(v.clone());
        on_stack.insert(v.clone());

        if let Some(neighbors) = adj.get(v) {
            for w in neighbors {
                if !indices.contains_key(w) {
                    self.strongconnect(w, adj, index, stack, on_stack, indices, lowlinks, sccs);
                    let min_low = std::cmp::min(lowlinks[v], lowlinks[w]);
                    lowlinks.insert(v.clone(), min_low);
                } else if on_stack.contains(w) {
                    let min_low = std::cmp::min(lowlinks[v], indices[w]);
                    lowlinks.insert(v.clone(), min_low);
                }
            }
        }

        if lowlinks[v] == indices[v] {
            let mut scc = Vec::new();
            loop {
                let w = stack.pop().unwrap();
                on_stack.remove(&w);
                scc.push(w.clone());
                if w == *v {
                    break;
                }
            }
            sccs.push(scc);
        }
    }
}
