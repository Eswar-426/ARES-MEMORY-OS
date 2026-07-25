use ares_core::types::node::SymbolSignature;
use ares_core::{NodeType, ProjectId};
use ares_store::repositories::graph::SqliteGraphRepository;
use ares_store::Store;
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct SymbolResolver {
    graph_repo: Arc<SqliteGraphRepository>,
}

impl SymbolResolver {
    pub fn new(store: Store) -> Self {
        Self {
            graph_repo: Arc::new(SqliteGraphRepository::new(store)),
        }
    }

    pub fn resolve_all(
        &self,
        project_id: &ProjectId,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let unresolved_nodes = self.graph_repo.get_unresolved_nodes(project_id)?;
        if unresolved_nodes.is_empty() {
            return Ok(0);
        }

        let resolved_count = AtomicUsize::new(0);

        unresolved_nodes.par_iter().for_each(|unresolved_node| {
            if let Some(prop_val) = unresolved_node.properties.get("signature") {
                if let Ok(signature) = serde_json::from_value::<SymbolSignature>(prop_val.clone()) {
                    // Fetch candidates by exact name first
                    if let Ok(candidates) = self
                        .graph_repo
                        .get_nodes_by_name(project_id, &signature.name)
                    {
                        // Filter out unresolved candidates
                        let mut candidates: Vec<_> = candidates
                            .into_iter()
                            .filter(|n| n.properties.get("unresolved").is_none())
                            .collect();

                        // ── Module-to-file resolution ──────────────
                        // mod foo; creates an unresolved node named "foo".
                        // The actual file node is named "foo.rs" (label)
                        // or has file_path ending in "foo.rs" / "foo/mod.rs".
                        if signature.symbol_type == NodeType::Module {
                            let mut file_matches = Vec::new();
                            for name_variant in
                                &[format!("{}.rs", signature.name), signature.name.clone()]
                            {
                                if let Ok(file_candidates) =
                                    self.graph_repo.get_nodes_by_name(project_id, name_variant)
                                {
                                    file_matches.extend(file_candidates.into_iter().filter(|n| {
                                        n.properties.get("unresolved").is_none()
                                            && n.node_type == NodeType::File
                                    }));
                                }
                            }
                            if !file_matches.is_empty() {
                                // Prepend file matches so they take priority over inline modules of the same name
                                file_matches.extend(candidates);
                                candidates = file_matches;
                            }
                        }
                        // ── End module-to-file resolution ──────────

                        // Sort candidates by proximity to the unresolved node's file path, if available
                        if candidates.len() > 1 {
                            if let Some(source_path) = &unresolved_node.file_path {
                                candidates.sort_by_key(|c| {
                                    if let Some(c_path) = &c.file_path {
                                        // Calculate shared prefix length (higher is better, so return negative for sorting)
                                        let source_parts: Vec<_> = source_path.split('/').collect();
                                        let c_parts: Vec<_> = c_path.split('/').collect();
                                        let mut match_len = 0;
                                        for (s, p) in source_parts.iter().zip(c_parts.iter()) {
                                            if s == p {
                                                match_len += 1;
                                            } else {
                                                break;
                                            }
                                        }
                                        -(match_len as i32)
                                    } else {
                                        0
                                    }
                                });
                            }
                        }
                        // ── Directory proximity sorting ──────────────
                        // When multiple candidates exist (e.g. multiple rust.rs files),
                        // prefer the one in the same directory as the declaring file.
                        if candidates.len() > 1 {
                            if let Some(declaring_file) = unresolved_node
                                .properties
                                .get("declaring_file")
                                .and_then(|v| v.as_str())
                            {
                                if !declaring_file.is_empty() {
                                    let declaring_dir = std::path::Path::new(declaring_file)
                                        .parent()
                                        .map(|p| p.to_string_lossy().to_string())
                                        .unwrap_or_default();
                                    candidates.sort_by(|a, b| {
                                        let a_same = a
                                            .file_path
                                            .as_ref()
                                            .and_then(|p| {
                                                std::path::Path::new(p).parent().map(|pp| {
                                                    pp.to_string_lossy().to_string()
                                                        == declaring_dir
                                                })
                                            })
                                            .unwrap_or(false);
                                        let b_same = b
                                            .file_path
                                            .as_ref()
                                            .and_then(|p| {
                                                std::path::Path::new(p).parent().map(|pp| {
                                                    pp.to_string_lossy().to_string()
                                                        == declaring_dir
                                                })
                                            })
                                            .unwrap_or(false);
                                        b_same.cmp(&a_same)
                                    });
                                }
                            }
                        }
                        // ── End directory proximity sorting ──────────

                        let mut best_match = if candidates.is_empty() {
                            None
                        } else if candidates.len() == 1 {
                            Some(candidates[0].clone())
                        } else {
                            None // Handled below
                        };

                        // ── Cross-crate resolution ────────────────────
                        // use ares_store::db::Something → find crates/ares-store/src/lib.rs
                        if best_match.is_none() && candidates.is_empty() {
                            let separators = ["::", "/"];
                            for sep in &separators {
                                if signature.name.contains(sep) {
                                    let crate_segment =
                                        signature.name.split(*sep).next().unwrap_or("");
                                    if !crate_segment.is_empty() {
                                        for variant in &[
                                            crate_segment.to_string(),
                                            crate_segment.replace("_", "-"),
                                        ] {
                                            if let Ok(matches) =
                                                self.graph_repo.get_nodes_by_path_fragment(variant)
                                            {
                                                let lib_matches: Vec<_> = matches
                                                    .into_iter()
                                                    .filter(|n| {
                                                        n.properties.get("unresolved").is_none()
                                                            && n.node_type == NodeType::File
                                                            && n.file_path
                                                                .as_ref()
                                                                .map(|fp| fp.ends_with("lib.rs"))
                                                                .unwrap_or(false)
                                                    })
                                                    .collect();
                                                if lib_matches.len() == 1 {
                                                    candidates = lib_matches;
                                                    break;
                                                }
                                            }
                                        }
                                        if !candidates.is_empty() {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        // ── End cross-crate resolution ────────────────

                        if best_match.is_none() && !candidates.is_empty() {
                            best_match = if candidates.len() == 1 {
                                Some(candidates[0].clone())
                            } else {
                                // Apply resolution priority

                                // 1. Exact module path
                                let by_mod = candidates.clone();
                                if let Some(_mod_path) = signature.module_path {
                                    // TODO: if graph nodes store module_path, check it. (Assuming they don't yet, skip or check file_path as proxy)
                                }

                                // 2. Exact file path
                                let mut by_file = by_mod.clone();
                                if let Some(ref file_path) = signature.file_path {
                                    by_file.retain(|n| n.file_path.as_ref() == Some(file_path));
                                }
                                if by_file.len() == 1 {
                                    Some(by_file[0].clone())
                                } else if !by_file.is_empty() {
                                    by_file.first().cloned()
                                } else {
                                    // 3. Exact type match
                                    let mut by_type = by_mod.clone();
                                    by_type.retain(|n| {
                                        n.node_type == signature.symbol_type
                                            || (signature.symbol_type == NodeType::Module
                                                && n.node_type == NodeType::File)
                                    });
                                    if by_type.len() == 1 {
                                        Some(by_type[0].clone())
                                    } else if !by_type.is_empty() {
                                        by_type.first().cloned()
                                    } else {
                                        candidates.first().cloned()
                                    }
                                }
                            };
                        }

                        if let Some(best) = best_match {
                            if self
                                .graph_repo
                                .redirect_edges(&unresolved_node.id, &best.id)
                                .is_ok()
                            {
                                let _ =
                                    self.graph_repo.delete_node_permanently(&unresolved_node.id);
                                resolved_count.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                }
            }
        });

        Ok(resolved_count.into_inner())
    }
}
