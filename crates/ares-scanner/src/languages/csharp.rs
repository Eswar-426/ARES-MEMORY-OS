use super::{ExtractionResult, LanguageExtractor};
use ares_core::{types::event::now_micros, GraphNode, NodeId, NodeType, ProjectId};
use tree_sitter::{Parser, Query, QueryCursor};

pub struct CSharpExtractor {
    query: Query,
}

impl CSharpExtractor {
    pub fn new() -> Self {
        let language = tree_sitter_c_sharp::language().into();
        let query_str = r#"
            (class_declaration name: (identifier) @name) @class
            (interface_declaration name: (identifier) @name) @interface
            (enum_declaration name: (identifier) @name) @enum
            (method_declaration name: (identifier) @name) @method
            (using_directive) @import
            (namespace_declaration name: (identifier) @name) @module
            
            (invocation_expression function: (identifier) @name) @call
            (invocation_expression function: (member_access_expression name: (identifier) @name)) @call
            (object_creation_expression type: (identifier) @name) @construct
        "#;
        let query = Query::new(&language, query_str).expect("Invalid C# Tree-sitter query");
        Self { query }
    }
}

impl Default for CSharpExtractor {
    fn default() -> Self {
        Self::new()
    }
}

struct ScopeDef {
    id: NodeId,
    start_line: usize,
    end_line: usize,
}

enum RefType {
    Call,
    Construct,
}

struct RefUse {
    name: String,
    ref_type: RefType,
    line: usize,
}

impl LanguageExtractor for CSharpExtractor {
    fn extract(
        &self,
        project_id: &ProjectId,
        file_node_id: &ares_core::NodeId,
        file_path: &str,
        source_code: &str,
    ) -> Result<ExtractionResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_c_sharp::language().into())?;

        let tree = match parser.parse(source_code, None) {
            Some(t) => t,
            None => return Err("Failed to parse C# source code".into()),
        };

        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        let mut scopes = Vec::new();
        let mut references = Vec::new();

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&self.query, tree.root_node(), source_code.as_bytes());

        let now = now_micros();

        for m in matches {
            let mut node_type_opt = None;
            let mut ref_type_opt = None;
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
                        "method" => Some(NodeType::Method),
                        "class" => Some(NodeType::Struct),
                        "interface" => Some(NodeType::Trait),
                        "enum" => Some(NodeType::Enum),
                        "module" => Some(NodeType::Module),
                        _ => None,
                    };

                    if node_type_opt.is_none() {
                        ref_type_opt = match capture_name {
                            "call" => Some(RefType::Call),
                            "construct" => Some(RefType::Construct),
                            _ => None,
                        };
                    }
                    start_line = capture.node.start_position().row + 1;
                    end_line = capture.node.end_position().row + 1;
                }
            }

            if !name.is_empty() {
                if let Some(node_type) = node_type_opt {
                    if node_type == NodeType::Tag && capture_names_contains_import(&m, &self.query)
                    {
                        let import_path = name
                            .replace("using", "")
                            .replace("static", "")
                            .replace(";", "")
                            .trim()
                            .to_string();
                        let unresolved_node_id =
                            ares_core::NodeId::from(format!("unresolved_{}", import_path));
                        let signature = ares_core::types::node::SymbolSignature {
                            name: import_path.clone(),
                            file_path: None,
                            module_path: None,
                            symbol_type: NodeType::Module,
                        };
                        let unresolved_node = GraphNode {
                            id: unresolved_node_id.clone(),
                            project_id: project_id.clone(),
                            node_type: NodeType::Module,
                            label: import_path.clone(),
                            properties: serde_json::json!({
                                "unresolved": true,
                                "signature": signature
                            }),
                            file_path: None,
                            created_at: now,
                            updated_at: now,
                            deleted_at: None,
                        };
                        nodes.push(unresolved_node);

                        let edge = ares_core::GraphEdge {
                            id: format!(
                                "edge_import_{}_{}",
                                file_node_id.as_str(),
                                unresolved_node_id.as_str()
                            ),
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
                        "language": "c-sharp"
                    });

                    let graph_node = GraphNode {
                        id: NodeId::new(),
                        project_id: project_id.clone(),
                        node_type,
                        label: name.clone(),
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

                    let reverse_edge = ares_core::GraphEdge {
                        id: format!(
                            "edge_containedin_{}_{}",
                            graph_node.id.as_str(),
                            file_node_id.as_str()
                        ),
                        project_id: project_id.clone(),
                        from_node_id: graph_node.id.clone(),
                        to_node_id: file_node_id.clone(),
                        edge_type: ares_core::EdgeType::ContainedIn,
                        weight: 1.0,
                        confidence: 1.0,
                        source: "scanner".to_string(),
                        valid_from: now,
                        valid_until: None,
                        created_at: now,
                    };

                    scopes.push(ScopeDef {
                        id: graph_node.id.clone(),
                        start_line,
                        end_line,
                    });

                    nodes.push(graph_node);
                    edges.push(edge);
                    edges.push(reverse_edge);
                } else if let Some(ref_type) = ref_type_opt {
                    references.push(RefUse {
                        name: name.clone(),
                        ref_type,
                        line: start_line,
                    });
                }
            }
        }

        // Process references to link them to the most specific enclosing scope
        for r in references {
            let mut best_scope_id = None;
            let mut min_size = usize::MAX;

            for scope in &scopes {
                if r.line >= scope.start_line && r.line <= scope.end_line {
                    let size = scope.end_line - scope.start_line;
                    if size < min_size {
                        min_size = size;
                        best_scope_id = Some(scope.id.clone());
                    }
                }
            }

            let source_node_id = best_scope_id.unwrap_or(file_node_id.clone());

            let edge_type = match r.ref_type {
                RefType::Call => ares_core::EdgeType::Calls,
                RefType::Construct => ares_core::EdgeType::Constructs,
            };

            let unresolved_node_id = ares_core::NodeId::from(format!("unresolved_{}", r.name));
            let expected_type = match r.ref_type {
                RefType::Call => NodeType::Method,
                RefType::Construct => NodeType::Struct,
            };

            let signature = ares_core::types::node::SymbolSignature {
                name: r.name.clone(),
                file_path: None,
                module_path: None,
                symbol_type: expected_type.clone(),
            };

            let unresolved_node = GraphNode {
                id: unresolved_node_id.clone(),
                project_id: project_id.clone(),
                node_type: expected_type,
                label: r.name.clone(),
                properties: serde_json::json!({
                    "unresolved": true,
                    "signature": signature
                }),
                file_path: None,
                created_at: now,
                updated_at: now,
                deleted_at: None,
            };
            nodes.push(unresolved_node);

            let ref_edge = ares_core::GraphEdge {
                id: format!(
                    "edge_{}_{}_{}",
                    edge_type.as_str(),
                    source_node_id.as_str(),
                    unresolved_node_id.as_str()
                ),
                project_id: project_id.clone(),
                from_node_id: source_node_id.clone(),
                to_node_id: unresolved_node_id.clone(),
                edge_type,
                weight: 1.0,
                confidence: 0.5,
                source: "scanner".to_string(),
                valid_from: now,
                valid_until: None,
                created_at: now,
            };
            edges.push(ref_edge);
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
    fn test_csharp_import_extraction() {
        let extractor = CSharpExtractor::new();
        let project_id = ProjectId::new();
        let file_node_id = ares_core::NodeId::new();

        let source_code = r#"
            using System.Collections.Generic;
            using Example.MyClass;
            
            namespace MyNamespace {
                class Main {
                    public void run() {
                        Console.WriteLine("Hello");
                        MyClass obj = new MyClass();
                    }
                }
            }
        "#;

        let result = extractor
            .extract(&project_id, &file_node_id, "src/Main.cs", source_code)
            .unwrap();

        let mut import_paths = Vec::new();
        for edge in &result.edges {
            if edge.edge_type == ares_core::EdgeType::Imports {
                import_paths.push(edge.source.replace("import:", ""));
            }
        }

        assert!(import_paths.contains(&"System.Collections.Generic".to_string()));
        assert!(import_paths.contains(&"Example.MyClass".to_string()));

        let mut class_found = false;
        let mut method_found = false;
        let mut namespace_found = false;
        for node in &result.nodes {
            if node.node_type == NodeType::Struct && node.label == "Main" {
                class_found = true;
            }
            if node.node_type == NodeType::Method && node.label == "run" {
                method_found = true;
            }
            if node.node_type == NodeType::Module && node.label == "MyNamespace" {
                namespace_found = true;
            }
        }

        assert!(class_found);
        assert!(method_found);
        assert!(namespace_found);
    }
}
