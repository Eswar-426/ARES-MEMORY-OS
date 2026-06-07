use ares_core::{AresError, ProjectId};
use ares_store::repositories::graph::SqliteGraphRepository;
use crate::extractor::ExtractorRouter;
use crate::hasher::hash_file;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

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

    pub fn scan_changed_files(&self, project_id: &ProjectId, changed_files: &[PathBuf]) -> Result<(), AresError> {
        self.scan_internal(project_id, Path::new(""), false, Some(changed_files.to_vec()))
    }

    pub fn scan_file(&self, project_id: &ProjectId, file_path: &Path) -> Result<(), AresError> {
        self.scan_internal(project_id, Path::new(""), false, Some(vec![file_path.to_path_buf()]))
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

        let files: Vec<PathBuf> = match specific_files {
            Some(f) => f,
            None => {
                let mut paths = Vec::new();
                for result in WalkBuilder::new(root_path).hidden(false).build() {
                    match result {
                        Ok(entry) => {
                            if entry.file_type().map_or(false, |ft| ft.is_file()) {
                                paths.push(entry.into_path());
                            }
                        }
                        Err(_) => continue,
                    }
                }
                paths
            }
        };

        let total = files.len() as u32;
        let parsed = AtomicU32::new(0);
        let failed = AtomicU32::new(0);

        files.par_iter().for_each(|path| {
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

            match self.extractor.extract(project_id, &path_str, &source_code) {
                Ok(Some(result)) => {
                    let mut node_ids = Vec::new();
                    for node in result.nodes {
                        node_ids.push(node.id.clone());
                        let _ = self.graph_repo.upsert_node(node);
                    }
                    for edge in result.edges {
                        let _ = self.graph_repo.upsert_edge(edge);
                    }
                    
                    let _ = self.graph_repo.update_scan_state(project_id, &path_str, &current_hash, &node_ids);
                    parsed.fetch_add(1, Ordering::Relaxed);
                }
                Ok(None) => {
                    // Not a supported language file
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
