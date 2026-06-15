pub mod architecture;
pub mod dependency;
pub mod neighbors;
pub mod shortest_path;

pub use architecture::*;
pub use dependency::*;
pub use neighbors::*;
pub use shortest_path::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraversalConfig {
    pub max_depth: usize,
    pub max_neighbors: usize,
    pub max_results: usize,
}

impl Default for TraversalConfig {
    fn default() -> Self {
        Self {
            max_depth: 5,
            max_neighbors: 100,
            max_results: 50,
        }
    }
}
