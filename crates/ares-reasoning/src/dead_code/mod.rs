use crate::graph::ReasoningGraph;
use crate::models::DeadCodeCandidate;
use ares_core::{EdgeType, NodeType};

pub struct DeadCodeAnalyzer;

impl DeadCodeAnalyzer {
    pub fn analyze(graph: &ReasoningGraph) -> Vec<DeadCodeCandidate> {
        let mut candidates = Vec::new();

        for (id, node) in &graph.nodes {
            if node.file_path.is_none() {
                continue;
            }
            if node.label.len() == 36 || node.label.starts_with("01") {
                continue; // likely a generated UUID
            }
            
            let incoming = graph.incoming.get(id);
            let label_lower = node.label.to_lowercase();
            
            // Skip common non-dead things
            if label_lower.contains("test") 
                || label_lower.contains("mock") 
                || label_lower == "main"
                || label_lower == "new"
                || label_lower == "default"
                || label_lower == "from"
                || label_lower == "into"
                || label_lower == "clone"
                || label_lower == "debug"
                || label_lower == "serialize"
                || label_lower == "deserialize"
                || label_lower.starts_with("unresolved_")
                || label_lower == "lib"
                || label_lower == "build"
            {
                continue;
            }

            // Check visibility
            let is_pub = node.properties.get("visibility").and_then(|v| v.as_str()).map_or(false, |s| s == "pub" || s == "public");
            let is_test = node.properties.get("is_test").and_then(|v| v.as_bool()).unwrap_or(false);
            if is_test {
                continue;
            }

            match node.node_type {
                NodeType::Function | NodeType::Method => {
                    let has_calls = incoming.map_or(false, |edges| {
                        edges.iter().any(|e| matches!(e.edge_type, EdgeType::Calls | EdgeType::Invokes | EdgeType::Uses))
                    });

                    let mut confidence = 0.0;
                    if !has_calls {
                        confidence = 100.0;
                        let has_refs = incoming.map_or(false, |edges| {
                            edges.iter().any(|e| matches!(e.edge_type, EdgeType::References))
                        });
                        if has_refs {
                            confidence -= 40.0;
                        }
                        if is_pub {
                            confidence -= 50.0; // Public APIs might be used outside
                        }
                    }

                    if confidence >= 50.0 { // only report if >= 50% confident
                        candidates.push(DeadCodeCandidate {
                            node_id: id.clone(),
                            label: node.label.clone(),
                            confidence,
                        });
                    }
                }
                NodeType::Struct | NodeType::Class | NodeType::Enum => {
                    let has_refs = incoming.map_or(false, |edges| {
                        edges.iter().any(|e| {
                            matches!(
                                e.edge_type,
                                EdgeType::References
                                    | EdgeType::Uses
                                    | EdgeType::Constructs
                                    | EdgeType::Extends
                            )
                        })
                    });
                    if !has_refs {
                        let conf = if is_pub { 50.0 } else { 100.0 };
                        if conf >= 50.0 {
                            candidates.push(DeadCodeCandidate {
                                node_id: id.clone(),
                                label: node.label.clone(),
                                confidence: conf,
                            });
                        }
                    }
                }
                NodeType::Trait | NodeType::Interface => {
                    let has_impls = incoming.map_or(false, |edges| {
                        edges.iter().any(|e| {
                            matches!(e.edge_type, EdgeType::Implements | EdgeType::UsesTrait)
                        })
                    });
                    if !has_impls && !is_pub {
                        candidates.push(DeadCodeCandidate {
                            node_id: id.clone(),
                            label: node.label.clone(),
                            confidence: 100.0,
                        });
                    }
                }
                NodeType::Module => {
                    let has_imports = incoming.map_or(false, |edges| {
                        edges.iter().any(|e| {
                            matches!(e.edge_type, EdgeType::Imports | EdgeType::UsesModule)
                        })
                    });
                    if !has_imports && label_lower != "main" && label_lower != "lib" {
                        let conf = if is_pub { 50.0 } else { 90.0 };
                        if conf >= 50.0 {
                            candidates.push(DeadCodeCandidate {
                                node_id: id.clone(),
                                label: node.label.clone(),
                                confidence: conf,
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        candidates
    }
}
