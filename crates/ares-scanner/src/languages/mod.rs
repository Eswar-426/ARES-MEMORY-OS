use ares_core::{GraphEdge, GraphNode, NodeId, ProjectId};

pub mod go;
pub mod javascript;
pub mod python;
pub mod rust;
pub mod typescript;

pub struct ExtractionResult {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

pub trait LanguageExtractor: Send + Sync {
    /// Attempt to parse the source code and extract nodes and edges.
    fn extract(
        &self,
        project_id: &ProjectId,
        file_node_id: &NodeId,
        file_path: &str,
        source_code: &str,
    ) -> Result<ExtractionResult, Box<dyn std::error::Error + Send + Sync>>;
}
