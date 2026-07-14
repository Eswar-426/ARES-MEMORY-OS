use super::{ExtractionResult, LanguageExtractor};
use ares_core::{types::event::now_micros, GraphEdge, GraphNode, NodeId, NodeType, ProjectId};
use tree_sitter::{Parser, Query, QueryCursor};

pub struct PhpExtractor {
    query: Option<Query>,
}

impl PhpExtractor {
    pub fn new() -> Self {
        let language = tree_sitter_php::LANGUAGE_PHP.into();
        let query_str = r#"
            (class_declaration name: (name) @name) @class
            (function_definition name: (name) @name) @function
            (method_declaration name: (name) @name) @function
            (use_declaration) @import
            (interface_declaration name: (name) @name) @class
            (trait_declaration name: (name) @name) @class
        "#;
        Self { query: super::try_build_query(language, query_str, "PHP") }
    }
}

impl Default for PhpExtractor {
    fn default() -> Self { Self::new() }
}

impl LanguageExtractor for PhpExtractor {
    fn extract(
        &self,
        project_id: &ProjectId,
        file_node_id: &NodeId,
        file_path: &str,
        source_code: &str,
    ) -> Result<ExtractionResult, Box<dyn std::error::Error + Send + Sync>> {
        let query = match &self.query {
            Some(q) => q,
            None => return Ok(ExtractionResult { nodes: Vec::new(), edges: Vec::new() }),
        };
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_php::LANGUAGE_PHP.into())?;
        let tree = parser.parse(source_code, None).ok_or("Failed to parse PHP source")?;

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let capture_names = query.capture_names();
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(query, tree.root_node(), source_code.as_bytes());
        let mut current_name: String = String::new();

        for m in matches {
            for capture in m.captures {
                let node = capture.node;
                match capture_names[capture.index as usize] {
                    "name" => { current_name = node.utf8_text(source_code.as_bytes())?.to_string(); }
                    "function" | "class" => {
                        let n_type = if capture_names[capture.index as usize] == "function" { NodeType::Function } else { NodeType::Class };
                        let id = NodeId::from(format!("{}_p_{}_{}", file_node_id.as_str(), current_name, node.start_position().row));
                        nodes.push(GraphNode { id: id.clone(), project_id: project_id.clone(), node_type: n_type, label: current_name.clone(), file_path: Some(file_path.to_string()), properties: serde_json::json!({"line": node.start_position().row, "column": node.start_position().column}), created_at: now_micros(), updated_at: now_micros(), deleted_at: None });
                        edges.push(GraphEdge { id: format!("{}_e_{}", id.as_str(), file_node_id.as_str()), project_id: project_id.clone(), from_node_id: file_node_id.clone(), to_node_id: id.clone(), edge_type: ares_core::EdgeType::Defines, weight: 1.0, confidence: 1.0, source: "scanner".to_string(), valid_from: now_micros(), valid_until: None, created_at: now_micros() });
                        current_name.clear();
                    }
                    "import" => {
                        let text = node.utf8_text(source_code.as_bytes())?.to_string();
                        let id = NodeId::from(format!("{}_p_imp_{}", file_node_id.as_str(), node.start_position().row));
                        nodes.push(GraphNode { id: id.clone(), project_id: project_id.clone(), node_type: NodeType::Tag, label: text.clone(), file_path: Some(file_path.to_string()), properties: serde_json::json!({"line": node.start_position().row, "column": node.start_position().column}), created_at: now_micros(), updated_at: now_micros(), deleted_at: None });
                        edges.push(GraphEdge { id: format!("{}_e_{}", id.as_str(), file_node_id.as_str()), project_id: project_id.clone(), from_node_id: file_node_id.clone(), to_node_id: id.clone(), edge_type: ares_core::EdgeType::Defines, weight: 1.0, confidence: 1.0, source: "scanner".to_string(), valid_from: now_micros(), valid_until: None, created_at: now_micros() });
                    }
                    _ => {}
                }
            }
        }
        Ok(ExtractionResult { nodes, edges })
    }
}
