use crate::extractor::ExtractorRouter;
use crate::hasher::hash_file;
use ares_core::{AresError, ProjectId};
use ares_store::repositories::graph::SqliteGraphRepository;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

pub struct Scanner {
    graph_repo: Arc<SqliteGraphRepository>,
    extractor: Arc<ExtractorRouter>,
}

impl Scanner {
    pub fn new(graph_repo: Arc<SqliteGraphRepository>) -> Self {
        Self {
            graph_repo,
            extractor: Arc::new(ExtractorRouter::new()),
        }
    }

    pub fn full_scan(&self, project_id: &ProjectId, root_path: &Path) -> Result<(), AresError> {
        self.scan_internal(project_id, root_path, true, None)
    }

    pub fn scan_project(&self, project_id: &ProjectId, root_path: &Path) -> Result<(), AresError> {
        self.scan_internal(project_id, root_path, false, None)
    }

    pub fn scan_changed_files(
        &self,
        project_id: &ProjectId,
        changed_files: &[PathBuf],
    ) -> Result<(), AresError> {
        self.scan_internal(
            project_id,
            Path::new(""),
            false,
            Some(changed_files.to_vec()),
        )
    }

    pub fn scan_file(&self, project_id: &ProjectId, file_path: &Path) -> Result<(), AresError> {
        self.scan_internal(
            project_id,
            Path::new(""),
            false,
            Some(vec![file_path.to_path_buf()]),
        )
    }

    fn scan_internal(
        &self,
        project_id: &ProjectId,
        root_path: &Path,
        force_full: bool,
        specific_files: Option<Vec<PathBuf>>,
    ) -> Result<(), AresError> {
        let run_type = if force_full { "full" } else { "incremental" };
        let run_id = self.graph_repo.start_scan_run(project_id, run_type)?;

        let mut paths = Vec::new();
        let mut dir_nodes: std::collections::HashMap<PathBuf, ares_core::NodeId> = std::collections::HashMap::new();

        if specific_files.is_none() {
            // Upsert Project node
            let project_node = ares_core::GraphNode {
                id: ares_core::NodeId::from(project_id.as_str()),
                project_id: project_id.clone(),
                node_type: ares_core::NodeType::Project,
                label: project_id.as_str().to_string(),
                properties: serde_json::json!({}),
                file_path: None,
                created_at: ares_core::types::event::now_micros(),
                updated_at: ares_core::types::event::now_micros(),
                deleted_at: None,
            };
            let _ = self.graph_repo.upsert_node(project_node);
            
            // The root directory gets mapped to the Project node ID
            dir_nodes.insert(root_path.to_path_buf(), ares_core::NodeId::from(project_id.as_str()));

            let walker = WalkBuilder::new(root_path)
                .hidden(false)
                .filter_entry(|e| {
                    let name = e.file_name().to_string_lossy();
                    !matches!(name.as_ref(), ".git" | "target" | ".gemini" | "artifacts"
                        | "node_modules" | "dist" | ".turbo" | ".ares" | "scratch")
                })
                .build();

            for result in walker {
                match result {
                    Ok(entry) => {
                        let path = entry.path().to_path_buf();
                        if entry.file_type().is_some_and(|ft| ft.is_dir()) {
                            if path == root_path {
                                continue;
                            }
                            
                            let dir_node_id = ares_core::NodeId::new();
                            let dir_node = ares_core::GraphNode {
                                id: dir_node_id.clone(),
                                project_id: project_id.clone(),
                                node_type: ares_core::NodeType::Folder,
                                label: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                                properties: serde_json::json!({}),
                                file_path: Some(path.to_string_lossy().to_string()),
                                created_at: ares_core::types::event::now_micros(),
                                updated_at: ares_core::types::event::now_micros(),
                                deleted_at: None,
                            };
                            let _ = self.graph_repo.upsert_node(dir_node);
                            
                            // Link to parent
                            if let Some(parent) = path.parent() {
                                if let Some(parent_id) = dir_nodes.get(parent) {
                                    let edge = ares_core::GraphEdge {
                                        id: format!("edge_contains_{}_{}", parent_id.as_str(), dir_node_id.as_str()),
                                        project_id: project_id.clone(),
                                        from_node_id: parent_id.clone(),
                                        to_node_id: dir_node_id.clone(),
                                        edge_type: ares_core::EdgeType::Contains,
                                        weight: 1.0,
                                        confidence: 1.0,
                                        source: "scanner".to_string(),
                                        valid_from: ares_core::types::event::now_micros(),
                                        valid_until: None,
                                        created_at: ares_core::types::event::now_micros(),
                                    };
                                    let _ = self.graph_repo.upsert_edge(edge);
                                }
                            }
                            
                            dir_nodes.insert(path, dir_node_id);
                        } else if entry.file_type().is_some_and(|ft| ft.is_file()) {
                            paths.push(path);
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
        
        let files: Vec<PathBuf> = match specific_files {
            Some(f) => f,
            None => paths,
        };

        let dir_nodes_arc = Arc::new(dir_nodes);
        let total = files.len() as u32;
        eprintln!("[scanner] Total files to process: {}", total);
        let parsed = AtomicU32::new(0);
        let failed = AtomicU32::new(0);

        files.iter().for_each(|path| {
            let done = parsed.load(Ordering::Relaxed) + failed.load(Ordering::Relaxed);
            if done % 50 == 0 {
                eprintln!("[scanner] Progress: {}/{}", done, total);
            }
            let path_str = path.to_string_lossy().to_string();

            let current_hash = match hash_file(path) {
                Ok(h) => h,
                Err(_) => {
                    failed.fetch_add(1, Ordering::Relaxed);
                    return;
                }
            };

            if !force_full {
                if let Ok(Some(prev_hash)) = self.graph_repo.get_scan_state(project_id, &path_str) {
                    if current_hash == prev_hash {
                        return; // Unchanged
                    }
                }
            }

            let source_code = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(_) => {
                    failed.fetch_add(1, Ordering::Relaxed);
                    return;
                }
            };

            let file_node_id = ares_core::NodeId::new();
            let now = ares_core::types::event::now_micros();
            let file_node = ares_core::GraphNode {
                id: file_node_id.clone(),
                project_id: project_id.clone(),
                node_type: ares_core::NodeType::File,
                label: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                properties: serde_json::json!({ "hash": current_hash }),
                file_path: Some(path_str.clone()),
                created_at: now,
                updated_at: now,
                deleted_at: None,
            };

            let mut edges_to_insert = Vec::new();

            if let Some(parent) = path.parent() {
                if let Some(parent_id) = dir_nodes_arc.get(parent) {
                    let edge = ares_core::GraphEdge {
                        id: format!("edge_contains_{}_{}", parent_id.as_str(), file_node_id.as_str()),
                        project_id: project_id.clone(),
                        from_node_id: parent_id.clone(),
                        to_node_id: file_node_id.clone(),
                        edge_type: ares_core::EdgeType::Contains,
                        weight: 1.0,
                        confidence: 1.0,
                        source: "scanner".to_string(),
                        valid_from: now,
                        valid_until: None,
                        created_at: now,
                    };
                    edges_to_insert.push(edge);
                }
            }

            match self.extractor.extract(project_id, &file_node_id, &path_str, &source_code) {
                Ok(Some(result)) => {
                    let mut node_ids = Vec::new();
                    
                    // Upsert the file node
                    node_ids.push(file_node_id.clone());
                    let _ = self.graph_repo.upsert_node(file_node);

                    for edge in edges_to_insert {
                        let _ = self.graph_repo.upsert_edge(edge);
                    }

                    for node in result.nodes {
                        node_ids.push(node.id.clone());
                        let _ = self.graph_repo.upsert_node(node);
                    }
                    for edge in result.edges {
                        let _ = self.graph_repo.upsert_edge(edge);
                    }

                    let _ = self.graph_repo.update_scan_state(
                        project_id,
                        &path_str,
                        &current_hash,
                        &node_ids,
                    );
                    parsed.fetch_add(1, Ordering::Relaxed);
                }
                Ok(None) => {
                    // Not a supported language file, but still save the file node
                    let node_ids = vec![file_node_id.clone()];
                    let _ = self.graph_repo.upsert_node(file_node);
                    
                    for edge in edges_to_insert {
                        let _ = self.graph_repo.upsert_edge(edge);
                    }

                    let _ = self.graph_repo.update_scan_state(
                        project_id,
                        &path_str,
                        &current_hash,
                        &node_ids,
                    );
                    parsed.fetch_add(1, Ordering::Relaxed);
                }
                Err(_) => {
                    failed.fetch_add(1, Ordering::Relaxed);
                }
            }
        });

        self.graph_repo.complete_scan_run(
            &run_id,
            "completed",
            total,
            parsed.load(Ordering::Relaxed),
            failed.load(Ordering::Relaxed),
        )?;

        Ok(())
    }
}
