use super::{ExtractionResult, LanguageExtractor};
use ares_core::{types::event::now_micros, GraphNode, NodeId, NodeType, ProjectId};
use tree_sitter::{Parser, Query, QueryCursor};

pub struct TypeScriptExtractor {
    query: Query,
}

impl TypeScriptExtractor {
    pub fn new() -> Self {
        let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
        let query_str = r#"
            (function_declaration name: (identifier) @name) @function
            (method_definition name: (property_identifier) @name) @method
            (class_declaration name: (type_identifier) @name) @class
            (interface_declaration name: (type_identifier) @name) @interface
            (enum_declaration name: (identifier) @name) @enum
        "#;
        let query = Query::new(&language, query_str).expect("Invalid TS Tree-sitter query");
        Self { query }
    }
}

impl Default for TypeScriptExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageExtractor for TypeScriptExtractor {
    fn extract(
        &self,
        project_id: &ProjectId,
        file_path: &str,
        source_code: &str,
    ) -> Result<ExtractionResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())?;

        let tree = match parser.parse(source_code, None) {
            Some(t) => t,
            None => return Err("Failed to parse TypeScript source code".into()),
        };

        let mut nodes = Vec::new();
        let edges = Vec::new();

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&self.query, tree.root_node(), source_code.as_bytes());

        let now = now_micros();

        for m in matches {
            let mut node_type_opt = None;
            let mut name = String::new();
            let mut start_line = 0;
            let mut end_line = 0;

            for capture in m.captures {
                let capture_name = self.query.capture_names()[capture.index as usize];
                let text = capture.node.utf8_text(source_code.as_bytes()).unwrap_or("");

                if capture_name == "name" {
                    name = text.to_string();
                } else {
                    node_type_opt = match capture_name {
                        "function" => Some(NodeType::Function),
                        "method" => Some(NodeType::Method),
                        "class" => Some(NodeType::Class),
                        "interface" => Some(NodeType::Interface),
                        "enum" => Some(NodeType::Enum),
                        _ => None,
                    };
                    start_line = capture.node.start_position().row + 1;
                    end_line = capture.node.end_position().row + 1;
                }
            }

            if let Some(node_type) = node_type_opt {
                if !name.is_empty() {
                    let properties = serde_json::json!({
                        "start_line": start_line,
                        "end_line": end_line,
                        "language": "typescript"
                    });

                    let graph_node = GraphNode {
                        id: NodeId::new(),
                        project_id: project_id.clone(),
                        node_type,
                        label: name,
                        properties,
                        file_path: Some(file_path.to_string()),
                        created_at: now,
                        updated_at: now,
                        deleted_at: None,
                    };
                    nodes.push(graph_node);
                }
            }
        }

        Ok(ExtractionResult { nodes, edges })
    }
}
