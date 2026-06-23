#![allow(unused_assignments)]
#![allow(dead_code)]
#![allow(clippy::needless_range_loop)]
use ares_knowledge_graph::models::{EdgeType, KnowledgeEdge, KnowledgeNode, NodeType};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub struct MarkdownIntelligenceExtractor;

impl MarkdownIntelligenceExtractor {
    pub fn extract_intelligence(
        files: &[PathBuf],
        _repo_root: &Path,
        code_artifacts: &[String], // All normalized code artifact paths
    ) -> (Vec<KnowledgeNode>, Vec<KnowledgeEdge>) {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        // Requirement -> Decision and Evidence -> Decision links need to be formed by finding REQ-xxx and ADR-xxx
        // Let's first build all documents and their identified IDs.

        struct DocMeta {
            id: String,
            path: String,
            node_type: NodeType,
            req_ids: Vec<String>,
            adr_ids: Vec<String>,
            mentioned_code: Vec<String>,
        }

        let mut docs = Vec::new();

        let valid_extensions = ["md", "adoc", "txt"];
        let valid_dirs = [
            "docs",
            "adr",
            "rfc",
            "requirements",
            "architecture",
            "decisions",
            "evidence",
        ];

        for file in files {
            let ext = file.extension().and_then(|s| s.to_str()).unwrap_or("");
            if !valid_extensions.contains(&ext) {
                continue;
            }

            let file_str = ares_core::canonicalize_node_id(&file.to_string_lossy());

            // Check if it's in a valid dir or root (since some ADRs might be in root, but let's be safe)
            let in_valid_dir = valid_dirs
                .iter()
                .any(|dir| file_str.starts_with(&format!("{}/", dir)))
                || file_str.contains("ADR")
                || file_str.contains("REQ");

            if !in_valid_dir && !file_str.ends_with(".md") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(file) {
                let mut node_type = NodeType::Evidence;
                let mut req_ids = Vec::new();
                let mut adr_ids = Vec::new();

                // Deterministic classification
                let file_upper = file_str.to_uppercase();
                let is_owner =
                    file_str.ends_with("CODEOWNERS") || file_str.ends_with("ownership.md");

                if is_owner {
                    node_type = NodeType::Owner;
                } else if file_str.contains("docs/requirements/")
                    || content.starts_with("# Requirement:")
                    || content.contains("Requirement: ")
                {
                    node_type = NodeType::Requirement;
                } else if file_str.contains("docs/decisions/")
                    || content.starts_with("# Decision:")
                    || content.contains("Decision: ")
                {
                    node_type = NodeType::Decision;
                } else if file_str.contains("docs/architecture/")
                    || content.starts_with("# Architecture")
                {
                    node_type = NodeType::Architecture;
                } else if file_str.contains("docs/evidence/")
                    || content.starts_with("# Benchmark")
                    || content.starts_with("# Evidence")
                    || content.contains("Test Result")
                    || content.contains("Investigation")
                {
                    node_type = NodeType::Evidence;
                } else if file_upper.contains("REQ-") {
                    node_type = NodeType::Requirement;
                } else if file_upper.contains("ADR-") {
                    node_type = NodeType::Decision;
                } else {
                    node_type = NodeType::Evidence;
                }

                // Extract IDs for cross-linking
                // Simple regex-less extraction for REQ-xxx and ADR-xxx
                for word in content.split_whitespace() {
                    if word.starts_with("REQ-") {
                        req_ids.push(
                            word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-')
                                .to_string(),
                        );
                    }
                    if word.starts_with("ADR-") {
                        adr_ids.push(
                            word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-')
                                .to_string(),
                        );
                    }
                }

                // Code references
                let mut mentioned_code = Vec::new();
                for code_path in code_artifacts {
                    if content.contains(code_path) {
                        mentioned_code.push(code_path.clone());
                    } else {
                        // Check if a parent directory is mentioned (e.g., "crates/ares-ingestion/")
                        let mut prefix_match = false;
                        let parts: Vec<&str> = code_path.split('/').collect();
                        let mut current_prefix = String::new();
                        for i in 0..parts.len() - 1 {
                            current_prefix.push_str(parts[i]);
                            current_prefix.push('/');
                            if content.contains(&current_prefix) {
                                prefix_match = true;
                                break;
                            }
                        }
                        if prefix_match {
                            mentioned_code.push(code_path.clone());
                        }
                    }
                }

                nodes.push(KnowledgeNode {
                    id: file_str.clone(),
                    node_type: node_type.clone(),
                    name: file
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    properties: serde_json::json!({"path": file_str}),
                    created_at: now,
                });

                docs.push(DocMeta {
                    id: file_str,
                    path: file.to_string_lossy().to_string(),
                    node_type,
                    req_ids,
                    adr_ids,
                    mentioned_code,
                });
            }
        }

        // Generate edges
        for doc in &docs {
            // Decision -> Code
            if doc.node_type == NodeType::Decision {
                for code in &doc.mentioned_code {
                    edges.push(KnowledgeEdge {
                        id: Uuid::new_v4().to_string(),
                        source_id: doc.id.clone(),
                        target_id: code.clone(),
                        edge_type: EdgeType::Drives,
                        confidence: 1.0,
                        created_at: now,
                        properties: serde_json::json!({}),
                    });
                }
            }

            // Requirement -> Code (if mentioned directly)
            if doc.node_type == NodeType::Requirement {
                for code in &doc.mentioned_code {
                    edges.push(KnowledgeEdge {
                        id: Uuid::new_v4().to_string(),
                        source_id: doc.id.clone(),
                        target_id: code.clone(),
                        edge_type: EdgeType::ImplementedBy,
                        confidence: 1.0,
                        created_at: now,
                        properties: serde_json::json!({}),
                    });
                }
            }

            // Cross-doc edges
            for other_doc in &docs {
                if doc.id == other_doc.id {
                    continue;
                }

                // If this is a Decision and it mentions another doc's REQ id -> Requirement -> Decision
                if doc.node_type == NodeType::Decision
                    && other_doc.node_type == NodeType::Requirement
                {
                    // Check if other_doc has a REQ ID that is mentioned in doc
                    let mut matched = false;
                    for req_id in &other_doc.req_ids {
                        if doc.req_ids.contains(req_id) {
                            matched = true;
                            break;
                        }
                    }
                    if matched
                        || doc
                            .req_ids
                            .iter()
                            .any(|r| other_doc.id.to_uppercase().contains(r))
                    {
                        edges.push(KnowledgeEdge {
                            id: Uuid::new_v4().to_string(),
                            source_id: other_doc.id.clone(),
                            target_id: doc.id.clone(),
                            edge_type: EdgeType::ResultsIn,
                            confidence: 1.0,
                            created_at: now,
                            properties: serde_json::json!({}),
                        });
                    }
                }

                // If this is Evidence and it mentions an ADR -> Evidence -> Decision
                if doc.node_type == NodeType::Evidence && other_doc.node_type == NodeType::Decision
                {
                    let mut matched = false;
                    for adr_id in &other_doc.adr_ids {
                        if doc.adr_ids.contains(adr_id) {
                            matched = true;
                            break;
                        }
                    }
                    if matched
                        || doc
                            .adr_ids
                            .iter()
                            .any(|a| other_doc.id.to_uppercase().contains(a))
                    {
                        edges.push(KnowledgeEdge {
                            id: Uuid::new_v4().to_string(),
                            source_id: doc.id.clone(),
                            target_id: other_doc.id.clone(),
                            edge_type: EdgeType::Supports,
                            confidence: 1.0,
                            created_at: now,
                            properties: serde_json::json!({}),
                        });
                    }
                }
            }
        }

        (nodes, edges)
    }
}
