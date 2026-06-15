use super::{ExtractionResult, LanguageExtractor};
use ares_core::{types::event::now_micros, GraphNode, NodeId, NodeType, ProjectId};
use tree_sitter::{Parser, Query, QueryCursor};

pub struct GoExtractor {
    query: Query,
}

impl GoExtractor {
    pub fn new() -> Self {
        let language = tree_sitter_go::LANGUAGE.into();
        let query_str = r#"
            (function_declaration name: (identifier) @name) @function
            (method_declaration name: (field_identifier) @name) @method
            (type_spec name: (type_identifier) @name type: (struct_type)) @struct
            (type_spec name: (type_identifier) @name type: (interface_type)) @interface
            (import_spec path: (interpreted_string_literal) @import)
        "#;
        let query = Query::new(&language, query_str).expect("Invalid Go Tree-sitter query");
        Self { query }
    }
}

impl Default for GoExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageExtractor for GoExtractor {
    fn extract(
        &self,
        project_id: &ProjectId,
        file_node_id: &ares_core::NodeId,
        file_path: &str,
        source_code: &str,
    ) -> Result<ExtractionResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_go::LANGUAGE.into())?;

        let tree = match parser.parse(source_code, None) {
            Some(t) => t,
            None => return Err("Failed to parse Go source code".into()),
        };

        let mut nodes = Vec::new();
        let mut edges = Vec::new();

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
                } else if capture_name == "import" {
                    node_type_opt = Some(NodeType::Tag);
                    name = text.to_string();
                } else {
                    node_type_opt = match capture_name {
                        "function" => Some(NodeType::Function),
                        "method" => Some(NodeType::Method),
                        "struct" => Some(NodeType::Struct),
                        "interface" => Some(NodeType::Interface),
                        _ => None,
                    };
                    start_line = capture.node.start_position().row + 1;
                    end_line = capture.node.end_position().row + 1;
                }
            }

            if let Some(node_type) = node_type_opt {
                if !name.is_empty() {
                    if node_type == NodeType::Tag && capture_names_contains_import(&m, &self.query) {
                        let import_path = name.replace("\"", "").trim().to_string();
                        let unresolved_node_id = ares_core::NodeId::from(format!("unresolved_{}", import_path));
                        let unresolved_node = GraphNode {
                            id: unresolved_node_id.clone(),
                            project_id: project_id.clone(),
                            node_type: NodeType::Module,
                            label: import_path.clone(),
                            properties: serde_json::json!({"unresolved": true}),
                            file_path: None,
                            created_at: now,
                            updated_at: now,
                            deleted_at: None,
                        };
                        nodes.push(unresolved_node);

                        let edge = ares_core::GraphEdge {
                            id: format!("edge_import_{}_{}", file_node_id.as_str(), unresolved_node_id.as_str()),
                            project_id: project_id.clone(),
                            from_node_id: file_node_id.clone(),
                            to_node_id: unresolved_node_id.clone(),
                            edge_type: ares_core::EdgeType::Imports,
                            weight: 1.0,
                            confidence: 0.5,
                            source: format!("import:{}", import_path),
                            valid_from: now,
                            valid_until: None,
                            created_at: now,
                        };
                        edges.push(edge);
                        continue;
                    }

                    let properties = serde_json::json!({
                        "start_line": start_line,
                        "end_line": end_line,
                        "language": "go"
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

                    let edge = ares_core::GraphEdge {
                        id: format!("edge_{}_{}", file_node_id.as_str(), graph_node.id.as_str()),
                        project_id: project_id.clone(),
                        from_node_id: file_node_id.clone(),
                        to_node_id: graph_node.id.clone(),
                        edge_type: ares_core::EdgeType::Defines,
                        weight: 1.0,
                        confidence: 1.0,
                        source: "scanner".to_string(),
                        valid_from: now,
                        valid_until: None,
                        created_at: now,
                    };

                    nodes.push(graph_node);
                    edges.push(edge);
                }
            }
        }

        Ok(ExtractionResult { nodes, edges })
    }
}

fn capture_names_contains_import(m: &tree_sitter::QueryMatch, query: &Query) -> bool {
    for capture in m.captures {
        if query.capture_names()[capture.index as usize] == "import" {
            return true;
        }
    }
    false
}
