use super::{ExtractionResult, LanguageExtractor};
use ares_core::{types::event::now_micros, GraphEdge, GraphNode, NodeId, NodeType, ProjectId};
use tree_sitter::{Parser, Query, QueryCursor};

pub struct KotlinExtractor {
    query: Option<Query>,
}

impl KotlinExtractor {
    pub fn new() -> Self {
        let language = tree_sitter_kotlin_ng::LANGUAGE.into();
        let query_str = r#"
            (class_declaration) @class
            (function_declaration) @function
            (object_declaration) @class
            (package_header) @module
            (import_header) @import
        "#;
        Self { query: super::try_build_query(language, query_str, "Kotlin") }
    }
}

impl Default for KotlinExtractor {
    fn default() -> Self { Self::new() }
}

fn extract_name_from_text(text: &str, keyword: &str) -> String {
    let rest = text.strip_prefix(keyword).unwrap_or(text);
    let rest = rest.trim_start();
    rest.chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

impl LanguageExtractor for KotlinExtractor {
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
        parser.set_language(&tree_sitter_kotlin_ng::LANGUAGE.into())?;
        let tree = parser.parse(source_code, None).ok_or("Failed to parse Kotlin source")?;

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let capture_names = query.capture_names();
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(query, tree.root_node(), source_code.as_bytes());

        for m in matches {
            for capture in m.captures {
                let node = capture.node;
                let text = node.utf8_text(source_code.as_bytes())?;
                
                let capture_name = capture_names[capture.index as usize];
                
                let mut node_type_opt = None;
                let mut name = String::new();
                
                match capture_name {
                    "function" => {
                        name = extract_name_from_text(text, "fun");
                        node_type_opt = Some(NodeType::Function);
                    }
                    "class" => {
                        name = if text.starts_with("object") {
                            extract_name_from_text(text, "object")
                        } else {
                            extract_name_from_text(text, "class")
                        };
                        node_type_opt = Some(NodeType::Class);
                    }
                    "module" => {
                        name = text.trim_start_matches("package").trim().to_string();
                        node_type_opt = Some(NodeType::Module);
                    }
                    "import" => {
                        name = text.to_string();
                        node_type_opt = Some(NodeType::Tag);
                    }
                    _ => {}
                }
                
                if let Some(n_type) = node_type_opt {
                    let id = NodeId::from(format!("{}_k_{}_{}", file_node_id.as_str(), name, node.start_position().row));
                    nodes.push(GraphNode {
                        id: id.clone(),
                        project_id: project_id.clone(),
                        node_type: n_type,
                        label: name.clone(),
                        file_path: Some(file_path.to_string()),
                        properties: serde_json::json!({
                            "line": node.start_position().row,
                            "column": node.start_position().column
                        }),
                        created_at: now_micros(),
                        updated_at: now_micros(),
                        deleted_at: None,
                    });
                    
                    edges.push(GraphEdge {
                        id: format!("{}_e_{}", id.as_str(), file_node_id.as_str()),
                        project_id: project_id.clone(),
                        from_node_id: file_node_id.clone(),
                        to_node_id: id.clone(),
                        edge_type: ares_core::EdgeType::Defines,
                        weight: 1.0,
                        confidence: 1.0,
                        source: "scanner".to_string(),
                        valid_from: now_micros(),
                        valid_until: None,
                        created_at: now_micros(),
                    });
                }
            }
        }
        Ok(ExtractionResult { nodes, edges })
    }
}
