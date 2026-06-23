use crate::models::{CaptureMethod, SourceProvenance};
use ares_core::{EdgeType, GraphEdge, GraphNode, NodeId, NodeType, ProjectId};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub struct CodeownersExtractor;

impl CodeownersExtractor {
    pub fn extract(
        project_path: &Path,
        project_id: &ProjectId,
        captured_at: i64,
    ) -> Result<(Vec<GraphNode>, Vec<GraphEdge>), String> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        let possible_paths = [
            project_path.join("CODEOWNERS"),
            project_path.join(".github").join("CODEOWNERS"),
            project_path.join("docs").join("CODEOWNERS"),
        ];

        let mut codeowners_content = None;
        for path in &possible_paths {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    codeowners_content = Some(content);
                    break;
                }
            }
        }

        let content = match codeowners_content {
            Some(c) => c,
            None => return Ok((vec![], vec![])),
        };

        let mut seen_persons = HashSet::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            let pattern = parts[0];
            let owners = &parts[1..];

            for owner in owners {
                let owner_clean = owner.trim_start_matches('@');
                let person_id = NodeId::from(format!("person:{}", owner_clean));

                let prov = SourceProvenance {
                    source_system: "codeowners".to_string(),
                    source_id: pattern.to_string(),
                    capture_method: CaptureMethod::Explicit,
                    captured_at,
                    confidence: CaptureMethod::Explicit.base_confidence(), // 1.0
                };

                let prov_val = serde_json::to_value(&prov).unwrap_or(serde_json::json!({}));

                if !seen_persons.contains(&person_id) {
                    seen_persons.insert(person_id.clone());

                    let mut props = serde_json::json!({
                        "name": owner_clean,
                        "email": owner_clean, // CODEOWNERS often uses username/team
                    });
                    if let Some(p) = props.as_object_mut() {
                        p.insert("provenance".to_string(), prov_val.clone());
                    }

                    nodes.push(GraphNode {
                        id: person_id.clone(),
                        project_id: project_id.clone(),
                        node_type: NodeType::Person,
                        label: owner_clean.to_string(),
                        properties: props,
                        file_path: None,
                        created_at: captured_at,
                        updated_at: captured_at,
                        deleted_at: None,
                    });
                }

                // Note: To be fully accurate, we'd need to resolve the glob `pattern` to specific files
                // For now, we will create a generic "OwnsPattern" or just link to the pattern string.
                // But the memory coverage engine expects `Owns` edges pointing to `File` nodes.
                // Resolving all files matching the pattern is expensive here.
                // In ARES, the scanner usually does this. Since this is an architectural correction,
                // we'll use the `ignore` crate to walk the directory and match the pattern,
                // or we can defer pattern matching to the `lib.rs` orchestrator.
                // Let's do a simple glob match for all files?
                // We'll leave it as creating the Person nodes for now, and handle resolving CODEOWNERS
                // to file edges properly with `ignore` crate.
            }
        }

        // We need to resolve patterns to actual files to create `Owns` edges.
        // We'll do a simple walk of the directory.
        let mut builder = ignore::WalkBuilder::new(project_path);
        builder.hidden(false).git_ignore(true);

        let walker = builder.build();
        for result in walker {
            if let Ok(entry) = result {
                if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                    let rel_path = entry
                        .path()
                        .strip_prefix(project_path)
                        .unwrap_or(entry.path());
                    let path_str = rel_path.to_string_lossy().replace('\\', "/");

                    // Simple pattern matching for CODEOWNERS
                    for line in content.lines() {
                        let line = line.trim();
                        if line.is_empty() || line.starts_with('#') {
                            continue;
                        }
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() < 2 {
                            continue;
                        }
                        let pattern = parts[0];
                        let owners = &parts[1..];

                        // Very basic glob matching (starts with, ends with, or exact)
                        // A true implementation would use the `glob` or `ignore` crate's override builder.
                        let matches = if pattern == "*" {
                            true
                        } else if pattern.starts_with('*') && pattern.ends_with('*') {
                            path_str.contains(&pattern[1..pattern.len() - 1])
                        } else if pattern.starts_with('*') {
                            path_str.ends_with(&pattern[1..])
                        } else if pattern.ends_with('*') {
                            path_str.starts_with(&pattern[..pattern.len() - 1])
                        } else if pattern.ends_with('/') {
                            path_str.starts_with(pattern)
                        } else {
                            path_str == pattern || path_str.starts_with(&format!("{}/", pattern))
                        };

                        if matches {
                            let file_node_id = ares_core::canonicalize_node_id(&path_str);
                            for owner in owners {
                                let owner_clean = owner.trim_start_matches('@');
                                let person_id = NodeId::from(format!("person:{}", owner_clean));

                                edges.push(GraphEdge {
                                    id: format!("{}-owns-{}", person_id.as_str(), file_node_id),
                                    project_id: project_id.clone(),
                                    from_node_id: person_id.clone(),
                                    to_node_id: NodeId::from(file_node_id.as_str()),
                                    edge_type: EdgeType::Owns,
                                    weight: 1.0,
                                    confidence: 1.0, // Explicit ownership
                                    source: "codeowners".to_string(),
                                    valid_from: captured_at,
                                    valid_until: None,
                                    created_at: captured_at,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok((nodes, edges))
    }
}
