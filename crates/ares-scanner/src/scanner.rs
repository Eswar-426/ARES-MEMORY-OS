#![allow(unused_imports)]
use crate::extractor::ExtractorRouter;
use crate::hasher::hash_file;
use ares_core::{AresError, ProjectId};
use ares_store::repositories::graph::SqliteGraphRepository;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// Skip files larger than this to prevent OOM during ingestion
const MAX_FILE_SIZE_BYTES: u64 = 2 * 1024 * 1024; // 2MB

/// Extensionless files that should still be scanned
const EXTENSIONLESS_FILES: &[&str] = &[
    "Makefile",
    "Dockerfile",
    "Jenkinsfile",
    "Vagrantfile",
    "CMakeLists.txt",
    "Gemfile",
    "Rakefile",
];
const IGNORED_FILES: &[&str] = &[
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "Cargo.lock",
];

const IGNORED_DIRS: &[&str] = &[
    ".git",
    "target",
    ".gemini",
    "artifacts",
    "node_modules",
    "dist",
    "out",
    "build",
    ".turbo",
    ".ares",
    "scratch",
    "cert_synthetic",
    "evaluation",
    "apps/dashboard",
];

fn should_scan_file(path: &Path) -> bool {
    // Check extensionless known files first
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if EXTENSIONLESS_FILES.contains(&name) {
            return true;
        }
    }

    // Check file size BEFORE reading
    if let Ok(meta) = path.metadata() {
        if meta.len() > MAX_FILE_SIZE_BYTES {
            return false;
        }
    } else {
        return false;
    }

    // Check extension
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("rs")
            | Some("ts")
            | Some("tsx")
            | Some("js")
            | Some("jsx")
            | Some("py")
            | Some("go")
            | Some("md")
            | Some("txt")
            | Some("json")
            | Some("yml")
            | Some("yaml")
            | Some("toml")
            | Some("sh")
            | Some("bash")
            | Some("xml")
            | Some("html")
            | Some("css")
            | Some("scss")
            | Some("sql")
            | Some("graphql")
    )
}

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

    pub fn full_scan(
        &self,
        project_id: &ProjectId,
        root_path: &Path,
    ) -> Result<crate::models::ScannerReport, AresError> {
        self.scan_internal(project_id, root_path, true, None)
    }

    pub fn scan_project(
        &self,
        project_id: &ProjectId,
        root_path: &Path,
    ) -> Result<crate::models::ScannerReport, AresError> {
        self.scan_internal(project_id, root_path, false, None)
    }

    pub fn scan_changed_files(
        &self,
        project_id: &ProjectId,
        changed_files: &[PathBuf],
    ) -> Result<crate::models::ScannerReport, AresError> {
        self.scan_internal(
            project_id,
            Path::new(""),
            false,
            Some(changed_files.to_vec()),
        )
    }

    pub fn scan_file(
        &self,
        project_id: &ProjectId,
        file_path: &Path,
    ) -> Result<crate::models::ScannerReport, AresError> {
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
    ) -> Result<crate::models::ScannerReport, AresError> {
        let run_type = if force_full { "full" } else { "incremental" };
        let run_id = self.graph_repo.start_scan_run(project_id, run_type)?;

        let mut paths = Vec::new();
        let mut dir_nodes: std::collections::HashMap<PathBuf, ares_core::NodeId> =
            std::collections::HashMap::new();

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
            dir_nodes.insert(
                root_path.to_path_buf(),
                ares_core::NodeId::from(project_id.as_str()),
            );

            let walker = WalkBuilder::new(root_path)
                .hidden(false)
                .filter_entry(|e| {
                    let name = e.file_name().to_string_lossy();
                    let path = e.path().to_string_lossy().replace("\\", "/");
                    !IGNORED_DIRS.iter().any(|&ignored| {
                        if ignored.contains('/') {
                            path.ends_with(ignored) || path.contains(&format!("/{}/", ignored)) || path.contains(&format!("/{}", ignored))
                        } else {
                            name == ignored
                        }
                    }) && !IGNORED_FILES.iter().any(|&ignored_file| name == ignored_file)
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
                                label: path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string(),
                                properties: serde_json::json!({}),
                                file_path: Some(ares_core::canonical_repo_path(
                                    &root_path.to_string_lossy(),
                                    &path.to_string_lossy(),
                                )),
                                created_at: ares_core::types::event::now_micros(),
                                updated_at: ares_core::types::event::now_micros(),
                                deleted_at: None,
                            };
                            let _ = self.graph_repo.upsert_node(dir_node);

                            // Link to parent
                            if let Some(parent) = path.parent() {
                                if let Some(parent_id) = dir_nodes.get(parent) {
                                    let edge = ares_core::GraphEdge {
                                        id: format!(
                                            "edge_contains_{}_{}",
                                            parent_id.as_str(),
                                            dir_node_id.as_str()
                                        ),
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
                        } else if entry.file_type().is_some_and(|ft| ft.is_file())
                            && should_scan_file(&path)
                        {
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
        let mut existing_files: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        if let Ok(nodes) = self.graph_repo.get_all_nodes(project_id) {
            for node in nodes {
                if let Some(file_path) = &node.file_path {
                    existing_files.insert(file_path.clone(), node.id.as_str().to_string());
                }
            }
        }

        let parsed = AtomicU32::new(0);
        let failed = AtomicU32::new(0);
        let skipped = AtomicU32::new(0);
        let symbols_extracted = AtomicU32::new(0);
        let imports_found = AtomicU32::new(0);
        let relationships_created = AtomicU32::new(0);
        let scanned_paths =
            std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashSet::new()));

        let ext_counts =
            std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));

        struct FileParseResult {
            file_node: ares_core::GraphNode,
            container_edges: Vec<ares_core::GraphEdge>,
            extracted_nodes: Vec<ares_core::GraphNode>,
            extracted_edges: Vec<ares_core::GraphEdge>,
            path_str: String,
            current_hash: String,
            node_ids: Vec<ares_core::NodeId>,
        }

        let parse_results: Vec<FileParseResult> = files
            .par_iter()
            .filter_map(|path| {
                let done = parsed.load(Ordering::Relaxed)
                    + failed.load(Ordering::Relaxed)
                    + skipped.load(Ordering::Relaxed);
                if done > 0 && done.is_multiple_of(50) {
                    eprintln!("[scanner] Progress: {}/{}", done, total);
                }

                let path_str = ares_core::canonical_repo_path(
                    &root_path.to_string_lossy(),
                    &path.to_string_lossy(),
                );

                let current_hash = match hash_file(path) {
                    Ok(h) => h,
                    Err(_) => {
                        failed.fetch_add(1, Ordering::Relaxed);
                        return None;
                    }
                };

                if !force_full {
                    if let Ok(Some(prev_hash)) =
                        self.graph_repo.get_scan_state(project_id, &path_str)
                    {
                        if current_hash == prev_hash {
                            let mut paths = scanned_paths.lock().unwrap();
                            paths.insert(path_str.clone());
                            skipped.fetch_add(1, Ordering::Relaxed);
                            return None; // Unchanged
                        }
                    }
                }

                let file_node_id = if let Some(existing_id) = existing_files.get(&path_str) {
                    ares_core::NodeId::from(existing_id.as_str())
                } else {
                    ares_core::NodeId::new()
                };

                let source_code = match std::fs::read_to_string(path) {
                    Ok(s) => s,
                    Err(_) => {
                        failed.fetch_add(1, Ordering::Relaxed);
                        return None;
                    }
                };

                let now = ares_core::types::event::now_micros();
                let file_node = ares_core::GraphNode {
                    id: file_node_id.clone(),
                    project_id: project_id.clone(),
                    node_type: ares_core::NodeType::File,
                    label: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    properties: serde_json::json!({ "hash": current_hash }),
                    file_path: Some(path_str.clone()),
                    created_at: now,
                    updated_at: now,
                    deleted_at: None,
                };

                {
                    let mut paths = scanned_paths.lock().unwrap();
                    paths.insert(path_str.clone());
                }

                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let mut counts = ext_counts.lock().unwrap();
                    *counts.entry(ext.to_string()).or_insert(0) += 1;
                }

                let mut container_edges = Vec::new();

                if let Some(parent) = path.parent() {
                    if let Some(parent_id) = dir_nodes_arc.get(parent) {
                        container_edges.push(ares_core::GraphEdge {
                            id: format!(
                                "edge_contains_{}_{}",
                                parent_id.as_str(),
                                file_node_id.as_str()
                            ),
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
                        });
                        container_edges.push(ares_core::GraphEdge {
                            id: format!(
                                "edge_containedin_{}_{}",
                                file_node_id.as_str(),
                                parent_id.as_str()
                            ),
                            project_id: project_id.clone(),
                            from_node_id: file_node_id.clone(),
                            to_node_id: parent_id.clone(),
                            edge_type: ares_core::EdgeType::ContainedIn,
                            weight: 1.0,
                            confidence: 1.0,
                            source: "scanner".to_string(),
                            valid_from: now,
                            valid_until: None,
                            created_at: now,
                        });
                    }
                }

                match self
                    .extractor
                    .extract(project_id, &file_node_id, &path_str, &source_code)
                {
                    Ok(Some(result)) => {
                        let imports = result
                            .edges
                            .iter()
                            .filter(|e| e.edge_type == ares_core::EdgeType::Imports)
                            .count();
                        imports_found.fetch_add(imports as u32, Ordering::Relaxed);
                        symbols_extracted.fetch_add(result.nodes.len() as u32, Ordering::Relaxed);
                        relationships_created.fetch_add(
                            (container_edges.len() + result.edges.len()) as u32,
                            Ordering::Relaxed,
                        );

                        let mut node_ids = vec![file_node_id.clone()];
                        node_ids.extend(result.nodes.iter().map(|n| n.id.clone()));

                        Some(FileParseResult {
                            file_node,
                            container_edges,
                            extracted_nodes: result.nodes,
                            extracted_edges: result.edges,
                            path_str,
                            current_hash,
                            node_ids,
                        })
                    }
                    Ok(None) => {
                        relationships_created
                            .fetch_add(container_edges.len() as u32, Ordering::Relaxed);
                        Some(FileParseResult {
                            file_node,
                            container_edges,
                            extracted_nodes: Vec::new(),
                            extracted_edges: Vec::new(),
                            path_str,
                            current_hash,
                            node_ids: vec![file_node_id.clone()],
                        })
                    }
                    Err(_) => {
                        failed.fetch_add(1, Ordering::Relaxed);
                        None
                    }
                }
            })
            .collect();

        // ═══════════════════════════════════════════════════════════
        //  PHASE 2: Sequential batch insert — single transaction
        // ═══════════════════════════════════════════════════════════
        {
            let mut conn = self
                .graph_repo
                .store()
                .get_conn()
                .expect("[scanner] Failed to get DB connection for batch insert");
            let tx = conn
                .transaction()
                .expect("[scanner] Failed to begin batch transaction");

            for result in &parse_results {
                // File node
                let _ = self
                    .graph_repo
                    .upsert_node_tx(&tx, result.file_node.clone());

                // Container edges (contains / contained_in)
                for edge in &result.container_edges {
                    let _ = self.graph_repo.upsert_edge_tx(&tx, edge.clone());
                }

                // Extracted symbols (structs, functions, traits, etc.)
                for node in &result.extracted_nodes {
                    let _ = self.graph_repo.upsert_node_tx(&tx, node.clone());
                }

                // Extracted relationships (imports, depends_on, etc.)
                for edge in &result.extracted_edges {
                    let _ = self.graph_repo.upsert_edge_tx(&tx, edge.clone());
                }
            }

            tx.commit()
                .expect("[scanner] Failed to commit batch transaction");
        }

        // Scan state updates (outside main transaction — low volume, uses existing method)
        for result in &parse_results {
            let _ = self.graph_repo.update_scan_state(
                project_id,
                &result.path_str,
                &result.current_hash,
                &result.node_ids,
            );
        }

        parsed.fetch_add(parse_results.len() as u32, Ordering::Relaxed);

        let p_count = parsed.load(Ordering::Relaxed);
        self.graph_repo.complete_scan_run(
            &run_id,
            "completed",
            total,
            p_count,
            failed.load(Ordering::Relaxed),
        )?;

        let current_time = ares_core::types::event::now_micros();
        let paths = scanned_paths.lock().unwrap();
        for (file_path, node_id) in existing_files {
            if !paths.contains(&file_path) {
                if let Ok(Some(mut node)) = self
                    .graph_repo
                    .get_node(&ares_core::NodeId::from(node_id.as_str()))
                {
                    node.deleted_at = Some(current_time);
                    let _ = self.graph_repo.upsert_node(node);
                }
            }
        }

        // Determine dominant language
        let mut dominant_lang = "Unknown".to_string();
        {
            let counts = ext_counts.lock().unwrap();
            let mut max_count = 0;
            for (ext, count) in counts.iter() {
                if *count > max_count {
                    max_count = *count;
                    dominant_lang = match ext.as_str() {
                        "rs" => "Rust",
                        "ts" => "TypeScript",
                        "js" => "JavaScript",
                        "py" => "Python",
                        "go" => "Go",
                        _ => "Unknown",
                    }
                    .to_string();
                }
            }
        }

        if dominant_lang != "Unknown" {
            if let Ok(Some(mut project_node)) = self
                .graph_repo
                .get_node(&ares_core::NodeId::from(project_id.as_str()))
            {
                let mut props = project_node
                    .properties
                    .as_object()
                    .cloned()
                    .unwrap_or_default();
                props.insert("language".to_string(), serde_json::json!(dominant_lang));
                project_node.properties = serde_json::Value::Object(props);
                let _ = self.graph_repo.upsert_node(project_node);
            }
        }

        let extraction_success_rate = if total > 0 {
            p_count as f64 / total as f64
        } else {
            1.0
        };

        // Run the SymbolResolver post-extraction
        let resolver = crate::resolver::SymbolResolver::new(self.graph_repo.store().clone());
        if let Ok(resolved) = resolver.resolve_all(project_id) {
            println!("Resolved {} symbols", resolved);
        }

        Ok(crate::models::ScannerReport {
            files_scanned: total as usize,
            parsed_files: p_count as usize,
            modules_scanned: dir_nodes_arc.len(),
            symbols_extracted: symbols_extracted.load(Ordering::Relaxed) as usize,
            imports_found: imports_found.load(Ordering::Relaxed) as usize,
            relationships_created: relationships_created.load(Ordering::Relaxed) as usize,
            extraction_success_rate,
        })
    }
}
