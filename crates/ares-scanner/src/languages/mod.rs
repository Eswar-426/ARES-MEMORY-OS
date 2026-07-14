use ares_core::{GraphEdge, GraphNode, NodeId, ProjectId};
use tree_sitter::Query;

pub mod cpp;
pub mod csharp;
pub mod php;
pub mod kotlin;
pub mod go;
pub mod java;
pub mod javascript;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod typescript;

/// Safely compile a tree-sitter query. Returns None and logs a warning
/// instead of panicking if the query syntax is invalid for this grammar version.
pub fn try_build_query(
    language: tree_sitter::Language,
    query_str: &str,
    lang_name: &str,
) -> Option<tree_sitter::Query> {
    match tree_sitter::Query::new(&language, query_str) {
        Ok(q) => Some(q),
        Err(e) => {
            eprintln!(
                "[ARES Scanner] Warning: Failed to compile {} tree-sitter query: {}. {} extraction disabled.",
                lang_name, e, lang_name
            );
            None
        }
    }
}

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
