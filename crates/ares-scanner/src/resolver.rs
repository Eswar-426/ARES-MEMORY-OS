use ares_core::types::node::SymbolSignature;
use ares_core::{NodeId, ProjectId};
use ares_store::Store;
use ares_store::repositories::graph::SqliteGraphRepository;
use std::sync::Arc;
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct SymbolResolver {
    graph_repo: Arc<SqliteGraphRepository>,
}

impl SymbolResolver {
    pub fn new(store: Store) -> Self {
        Self {
            graph_repo: Arc::new(SqliteGraphRepository::new(store)),
        }
    }

    pub fn resolve_all(&self, project_id: &ProjectId) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let unresolved_nodes = self.graph_repo.get_unresolved_nodes(project_id)?;
        if unresolved_nodes.is_empty() {
            return Ok(0);
        }

        let resolved_count = AtomicUsize::new(0);

        unresolved_nodes.par_iter().for_each(|unresolved_node| {
            if let Some(prop_val) = unresolved_node.properties.get("signature") {
                if let Ok(signature) = serde_json::from_value::<SymbolSignature>(prop_val.clone()) {
                    
                    // Fetch candidates by exact name first
                    if let Ok(candidates) = self.graph_repo.get_nodes_by_name(project_id, &signature.name) {
                        // Filter out unresolved candidates
                        let mut candidates: Vec<_> = candidates.into_iter()
                            .filter(|n| n.properties.get("unresolved").is_none())
                            .collect();

                        let best_match = if candidates.is_empty() {
                            None
                        } else if candidates.len() == 1 {
                            Some(candidates[0].clone())
                        } else {
                            // Apply resolution priority
                            
                            // 1. Exact module path
                            let mut by_mod = candidates.clone();
                            if let Some(ref mod_path) = signature.module_path {
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
                                // 3. Exact symbol type
                                let mut by_type = by_file.clone();
                                by_type.retain(|n| n.node_type == signature.symbol_type);
                                if !by_type.is_empty() {
                                    Some(by_type[0].clone())
                                } else {
                                    Some(by_file[0].clone())
                                }
                            } else {
                                // Fallback to type
                                let mut by_type = candidates.clone();
                                by_type.retain(|n| n.node_type == signature.symbol_type);
                                if !by_type.is_empty() {
                                    Some(by_type[0].clone())
                                } else {
                                    // 4. Exact name
                                    Some(candidates[0].clone())
                                }
                            }
                        };

                        if let Some(best) = best_match {
                            if self.graph_repo.redirect_edges(&unresolved_node.id, &best.id).is_ok() {
                                let _ = self.graph_repo.delete_node_permanently(&unresolved_node.id);
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
