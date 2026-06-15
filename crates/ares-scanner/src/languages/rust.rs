use super::{ExtractionResult, LanguageExtractor};
use ares_core::{types::event::now_micros, GraphNode, NodeId, NodeType, ProjectId};
use tree_sitter::{Parser, Query, QueryCursor};

pub struct RustExtractor {
    query: Query,
}

impl RustExtractor {
    pub fn new() -> Self {
        let language = tree_sitter_rust::LANGUAGE.into();
        let query_str = r#"
            (function_item name: (identifier) @name) @function
            (struct_item name: (type_identifier) @name) @struct
            (enum_item name: (type_identifier) @name) @enum
            (trait_item name: (type_identifier) @name) @trait
            (impl_item body: (declaration_list (function_item name: (identifier) @name) @method))
            (mod_item name: (identifier) @name) @module
            (use_declaration) @import
        "#;
        let query = Query::new(&language, query_str).expect("Invalid Rust Tree-sitter query");
        Self { query }
    }
}

impl Default for RustExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageExtractor for RustExtractor {
    fn extract(
        &self,
        project_id: &ProjectId,
        file_node_id: &ares_core::NodeId,
        file_path: &str,
        source_code: &str,
    ) -> Result<ExtractionResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;

        let tree = match parser.parse(source_code, None) {
            Some(t) => t,
            None => return Err("Failed to parse Rust source code".into()),
        };

        let mut nodes = Vec::new();
        let mut edges = Vec::new(); // Edges (e.g., calls) can be added later

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
                    node_type_opt = Some(NodeType::Tag); // Dummy to differentiate logic below
                    name = text.to_string();
                } else {
                    node_type_opt = match capture_name {
                        "function" => Some(NodeType::Function),
                        "method" => Some(NodeType::Method),
                        "class" | "struct" => Some(NodeType::Struct),
                        "enum" => Some(NodeType::Enum),
                        "trait" => Some(NodeType::Trait),
                        "module" => Some(NodeType::Module),
                        _ => None,
                    };
                    start_line = capture.node.start_position().row + 1;
                    end_line = capture.node.end_position().row + 1;
                }
            }

            if let Some(node_type) = node_type_opt {
                if !name.is_empty() {
                    if node_type == NodeType::Tag && capture_names_contains_import(&m, &self.query) {
                        // This is an import
                        let import_path = name.replace("use ", "").replace(";", "").trim().to_string();
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
                        "language": "rust"
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

#[cfg(test)]
mod tests {
    use super::*;
    use ares_core::ProjectId;

    #[test]
    fn test_rust_import_extraction() {
        let extractor = RustExtractor::new();
        let project_id = ProjectId::new();
        let file_node_id = ares_core::NodeId::new();
        
        let source_code = r#"
            use std::collections::HashMap;
            use crate::memory::builder;
            
            fn main() {}
        "#;
        
        let result = extractor.extract(&project_id, &file_node_id, "src/main.rs", source_code).unwrap();
        
        // Should have 1 function node and 2 unresolved module nodes
        assert_eq!(result.nodes.len(), 3);
        
        // Should have 1 Defines edge and 2 Imports edges
        assert_eq!(result.edges.len(), 3);
        
        let mut import_paths = Vec::new();
        for edge in result.edges {
            if edge.edge_type == ares_core::EdgeType::Imports {
                import_paths.push(edge.source.replace("import:", ""));
            }
        }
        
        assert_eq!(import_paths.len(), 2);
        assert!(import_paths.contains(&"std::collections::HashMap".to_string()));
        assert!(import_paths.contains(&"crate::memory::builder".to_string()));
    }
}
