use crate::extractors::markdown::MarkdownIntelligenceExtractor;
use crate::extractors::ownership::OwnershipExtractor;
use crate::extractors::rust::RustDependencyExtractor;
use crate::extractors::tests::TestResolutionEngine;
use crate::extractors::typescript::TypeScriptDependencyExtractor;
use crate::scanner::RepositoryScanner;
use ares_knowledge_graph::models::GraphEvent;
use ares_knowledge_graph::models::{EdgeType, KnowledgeEdge, KnowledgeNode, NodeType};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub struct GraphBuilder {
    root: PathBuf,
    incremental_files: Option<Vec<PathBuf>>,
}

impl GraphBuilder {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            incremental_files: None,
        }
    }

    pub fn set_incremental_files(&mut self, files: Vec<PathBuf>) {
        self.incremental_files = Some(files);
    }

    pub fn build<F>(&self, mut sink: F) -> Result<(), ares_core::AresError>
    where
        F: FnMut(GraphEvent) -> Result<(), ares_core::AresError>,
    {
        let scanner = RepositoryScanner::new(&self.root);
        let mut files = scanner.scan();
        if let Some(inc_files) = &self.incremental_files {
            // Filter files to only those that match the incremental files.
            // Since incremental files might be absolute, we ensure we match exactly.
            let canonical_inc: Vec<String> = inc_files
                .iter()
                .map(|p| p.to_string_lossy().to_string().replace('\\', "/"))
                .collect();

            files.retain(|f| {
                let fs = f.to_string_lossy().to_string().replace('\\', "/");
                canonical_inc
                    .iter()
                    .any(|inc| fs.ends_with(inc) || inc.ends_with(&fs))
            });
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ares_core::AresError::validation(format!("System time error: {}", e)))?
            .as_millis() as i64;

        // Add Repository Node
        let repo_id = "REPO-ROOT".to_string();
        sink(GraphEvent::Node(KnowledgeNode {
            id: repo_id.clone(),
            node_type: NodeType::Repository,
            name: self
                .root
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            properties: serde_json::json!({"path": self.root.to_string_lossy()}),
            created_at: now,
        }))?;

        let mut code_artifact_ids = Vec::new();

        // Add CodeArtifacts
        for file in &files {
            let file_str = ares_core::canonicalize_node_id(&file.to_string_lossy());
            code_artifact_ids.push(file_str.clone());

            sink(GraphEvent::Node(KnowledgeNode {
                id: file_str.clone(),
                node_type: NodeType::CodeArtifact,
                name: file
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                properties: serde_json::json!({"path": file_str}),
                created_at: now,
            }))?;

            sink(GraphEvent::Edge(KnowledgeEdge {
                id: Uuid::new_v4().to_string(),
                source_id: repo_id.clone(),
                target_id: file_str.clone(),
                edge_type: EdgeType::Contains,
                confidence: 1.0,
                created_at: now,
                properties: serde_json::json!({}),
            }))?;
        }

        let (int_nodes, int_edges) = MarkdownIntelligenceExtractor::extract_intelligence(
            &files,
            &self.root,
            &code_artifact_ids,
        );
        for n in int_nodes {
            sink(GraphEvent::Node(n))?;
        }
        for e in int_edges {
            sink(GraphEvent::Edge(e))?;
        }

        // Ownership
        let ownerships = OwnershipExtractor::extract_ownership(&self.root);
        let mut owner_map = std::collections::HashSet::new();
        for (_, owner) in &ownerships {
            owner_map.insert(owner.clone());
        }
        for owner in owner_map {
            sink(GraphEvent::Node(KnowledgeNode {
                id: owner.clone(),
                node_type: NodeType::Owner,
                name: owner.clone(),
                properties: serde_json::json!({}),
                created_at: now,
            }))?;
        }

        let mut synthesized_nodes = std::collections::HashSet::new();

        // Map ownership to files
        for file in &files {
            let file_str = ares_core::canonicalize_node_id(&file.to_string_lossy());
            for (pattern, owner) in &ownerships {
                let canonical_pattern = ares_core::canonicalize_node_id(pattern);
                if file_str.contains(&canonical_pattern) || pattern == "*" {
                    if synthesized_nodes.insert(owner.clone()) {
                        sink(GraphEvent::Node(KnowledgeNode {
                            id: owner.clone(),
                            node_type: NodeType::Owner,
                            name: owner.clone(),
                            properties: serde_json::json!({"pattern": canonical_pattern}),
                            created_at: now,
                        }))?;
                    }

                    sink(GraphEvent::Edge(KnowledgeEdge {
                        id: Uuid::new_v4().to_string(),
                        source_id: file_str.clone(),
                        target_id: owner.clone(),
                        edge_type: EdgeType::OwnedBy,
                        confidence: 1.0,
                        created_at: now,
                        properties: serde_json::json!({}),
                    }))?;
                }
            }
        }

        // Tests
        let test_relations = TestResolutionEngine::extract_test_relations(&files);
        for (code, test) in test_relations {
            let code_str = ares_core::canonicalize_node_id(&code.to_string_lossy());
            let test_str = ares_core::canonicalize_node_id(&test.to_string_lossy());

            // To be robust, ensure we emit nodes for the tests and code themselves.
            // Often they are already emitted via walkdir, but just in case:
            if synthesized_nodes.insert(code_str.clone()) {
                sink(GraphEvent::Node(KnowledgeNode {
                    id: code_str.clone(),
                    node_type: NodeType::CodeArtifact,
                    name: code
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    properties: serde_json::json!({}),
                    created_at: now,
                }))?;
            }
            if synthesized_nodes.insert(test_str.clone()) {
                sink(GraphEvent::Node(KnowledgeNode {
                    id: test_str.clone(),
                    node_type: NodeType::Test,
                    name: test
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    properties: serde_json::json!({}),
                    created_at: now,
                }))?;
            }

            sink(GraphEvent::Edge(KnowledgeEdge {
                id: Uuid::new_v4().to_string(),
                source_id: code_str,
                target_id: test_str,
                edge_type: EdgeType::ValidatedBy,
                confidence: 1.0,
                created_at: now,
                properties: serde_json::json!({}),
            }))?;
        }

        // Dependencies
        for file in &files {
            let file_str = ares_core::canonicalize_node_id(&file.to_string_lossy());
            if file_str.ends_with("Cargo.toml") {
                let deps = RustDependencyExtractor::extract_dependencies(file);
                for (source_file, dep_name) in deps {
                    let dep_id = format!("DEP-RUST-{}", dep_name);

                    if synthesized_nodes.insert(dep_id.clone()) {
                        sink(GraphEvent::Node(KnowledgeNode {
                            id: dep_id.clone(),
                            node_type: NodeType::CodeArtifact,
                            name: dep_name.clone(),
                            properties: serde_json::json!({"type": "rust_dependency"}),
                            created_at: now,
                        }))?;
                    }

                    sink(GraphEvent::Edge(KnowledgeEdge {
                        id: Uuid::new_v4().to_string(),
                        source_id: ares_core::canonicalize_node_id(&source_file),
                        target_id: dep_id,
                        edge_type: EdgeType::DependsOn,
                        confidence: 1.0,
                        created_at: now,
                        properties: serde_json::json!({"type": "rust"}),
                    }))?;
                }
            } else if file_str.ends_with("package.json") {
                let deps = TypeScriptDependencyExtractor::extract_dependencies(file);
                for (source_file, dep_name) in deps {
                    let dep_id = format!("DEP-TS-{}", dep_name);

                    if synthesized_nodes.insert(dep_id.clone()) {
                        sink(GraphEvent::Node(KnowledgeNode {
                            id: dep_id.clone(),
                            node_type: NodeType::CodeArtifact,
                            name: dep_name.clone(),
                            properties: serde_json::json!({"type": "typescript_dependency"}),
                            created_at: now,
                        }))?;
                    }

                    sink(GraphEvent::Edge(KnowledgeEdge {
                        id: Uuid::new_v4().to_string(),
                        source_id: ares_core::canonicalize_node_id(&source_file),
                        target_id: dep_id,
                        edge_type: EdgeType::DependsOn,
                        confidence: 1.0,
                        created_at: now,
                        properties: serde_json::json!({"type": "typescript"}),
                    }))?;
                }
            }
        }

        // GapGenerator needs a rewrite to operate on database instead of memory vectors.
        // We skip memory GapGenerator here and rely on the database projection step.

        Ok(())
    }
}
