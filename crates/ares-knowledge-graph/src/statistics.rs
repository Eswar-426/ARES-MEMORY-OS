use ares_core::AresError;
use ares_store::Store;

use petgraph::graph::UnGraph;
use petgraph::visit::Bfs;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct GraphStatistics {
    pub node_count: usize,
    pub edge_count: usize,
    pub connected_components: usize,
    pub largest_component: usize,
    pub average_degree: f64,
    pub max_depth: usize,
}

pub fn compute_statistics(store: &Store) -> Result<GraphStatistics, AresError> {
    let conn = store.get_conn()?;

    // Fetch edges to build petgraph
    let mut stmt = conn
        .prepare("SELECT from_node_id, to_node_id FROM graph_edges WHERE valid_until IS NULL")
        .map_err(|e| AresError::Io(std::io::Error::other(e.to_string())))?;

    let edge_iter = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| AresError::Io(std::io::Error::other(e.to_string())))?;

    let mut graph = UnGraph::<String, ()>::new_undirected();
    let mut nodes = HashMap::new();
    let mut node_count = 0;
    let mut edge_count = 0;

    for edge_res in edge_iter {
        let (from, to) =
            edge_res.map_err(|e| AresError::Io(std::io::Error::other(e.to_string())))?;

        let from_idx = *nodes.entry(from.clone()).or_insert_with(|| {
            let idx = graph.add_node(from);
            node_count += 1;
            idx
        });

        let to_idx = *nodes.entry(to.clone()).or_insert_with(|| {
            let idx = graph.add_node(to);
            node_count += 1;
            idx
        });

        graph.add_edge(from_idx, to_idx, ());
        edge_count += 1;
    }

    if node_count == 0 {
        return Ok(GraphStatistics::default());
    }

    // Compute Connected Components and Largest Component using non-recursive BFS
    let mut visited = std::collections::HashSet::new();
    let mut connected_comps = 0;
    let mut largest_comp = 0;

    for node in graph.node_indices() {
        if !visited.contains(&node) {
            connected_comps += 1;
            let mut current_comp_size = 0;
            let mut bfs = Bfs::new(&graph, node);
            while let Some(nx) = bfs.next(&graph) {
                if visited.insert(nx) {
                    current_comp_size += 1;
                }
            }
            if current_comp_size > largest_comp {
                largest_comp = current_comp_size;
            }
        }
    }

    // Estimate Max Depth via BFS from a few random nodes
    let mut max_depth = 0;

    // For large graphs, exact max depth (diameter) is O(V*(V+E)). We approximate by doing a BFS from a few nodes.
    for start_node in graph.node_indices().take(10) {
        let mut bfs = Bfs::new(&graph, start_node);
        let mut depth_map = HashMap::new();
        depth_map.insert(start_node, 0);

        while let Some(nx) = bfs.next(&graph) {
            let current_depth = *depth_map.get(&nx).unwrap_or(&0);
            max_depth = max_depth.max(current_depth);

            for neighbor in graph.neighbors(nx) {
                depth_map.entry(neighbor).or_insert(current_depth + 1);
            }
        }
    }

    Ok(GraphStatistics {
        node_count,
        edge_count,
        connected_components: connected_comps,
        largest_component: largest_comp,
        average_degree: if node_count > 0 {
            (edge_count as f64) * 2.0 / (node_count as f64)
        } else {
            0.0
        }, // *2 for undirected degree? Actually typical degree is 2E/V.
        max_depth,
    })
}
