use super::{ExtractionResult, LanguageExtractor};
use ares_core::{types::event::now_micros, GraphNode, NodeId, NodeType, ProjectId};
use tree_sitter::{Parser, Query, QueryCursor};

pub struct PythonExtractor {
    query: Query,
}

impl PythonExtractor {
    pub fn new() -> Self {
        let language = tree_sitter_python::LANGUAGE.into();
        let query_str = r#"
            (function_definition name: (identifier) @name) @function
            (class_definition name: (identifier) @name) @class
        "#;
        let query = Query::new(&language, query_str).expect("Invalid Python Tree-sitter query");
        Self { query }
    }
}

impl Default for PythonExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageExtractor for PythonExtractor {
    fn extract(
        &self,
        project_id: &ProjectId,
        file_path: &str,
        source_code: &str,
    ) -> Result<ExtractionResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_python::LANGUAGE.into())?;

        let tree = match parser.parse(source_code, None) {
            Some(t) => t,
            None => return Err("Failed to parse Python source code".into()),
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
                        "class" => Some(NodeType::Class),
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
                        "language": "python"
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
