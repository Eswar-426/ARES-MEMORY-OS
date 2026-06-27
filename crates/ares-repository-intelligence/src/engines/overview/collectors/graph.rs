use crate::engines::overview::models::GraphOverview;
use ares_store::Store;

pub async fn collect(store: &Store) -> GraphOverview {
    let stats = store.overview_graph_stats().unwrap_or_else(|_| ares_store::overview::GraphStats {
        nodes: 0,
        edges: 0,
        files: 0,
        directories: 0,
        commits: 0,
        authors: 0,
    });
        
    let average_degree = if stats.nodes > 0 { stats.edges as f32 / stats.nodes as f32 } else { 0.0 };
    let graph_density = if stats.nodes > 1 { stats.edges as f32 / (stats.nodes as f32 * (stats.nodes as f32 - 1.0)) } else { 0.0 };
    
    let largest_component = (stats.nodes as f32 * 0.8) as usize; // Placeholder for expensive SCC computation
    let depth = 5; // Placeholder for path depth

    GraphOverview {
        nodes: stats.nodes,
        edges: stats.edges,
        files: stats.files,
        directories: stats.directories,
        commits: stats.commits,
        authors: stats.authors,
        average_degree,
        graph_density,
        largest_component,
        depth,
    }
}
