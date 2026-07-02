use crate::models::{
    ContributorRef, DependencyRef, EngineeringEvidence, EngineeringQuery, EntityMetrics, EntityRef,
    GitEvidence, Timestamps,
};
use ares_core::id::NodeId;
use ares_core::AresError;
use ares_store::repositories::graph::SqliteGraphRepository;
use ares_store::Store;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════
// EvidenceService — the universal data fetcher
// ═══════════════════════════════════════════════════════════════════

pub struct EvidenceService {
    store: Store,
}

impl EvidenceService {
    fn is_test_entity(label: &str, file_path: &Option<String>, node_type: &str) -> bool {
        if node_type == "test" {
            return true;
        }
        let name = file_path.as_deref().unwrap_or(label);
        name.contains("/tests/")
            || name.contains("\\tests\\")
            || name.ends_with("_test.py")
            || name.ends_with("_test.rs")
            || name.ends_with("_test.ts")
            || name.ends_with("_test.js")
            || name.starts_with("test_")
            || label.starts_with("test_")
    }
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    fn resolve_node_id(
        &self,
        repo: &SqliteGraphRepository,
        raw_id: &str,
    ) -> Result<String, AresError> {
        if raw_id.len() == 36 && raw_id.chars().filter(|&c| c == '-').count() == 4 {
            return Ok(raw_id.to_string());
        }
        repo.get_id_by_path(raw_id)
    }

    pub fn collect(&self, query: &EngineeringQuery) -> Result<EngineeringEvidence, AresError> {
        let repo = SqliteGraphRepository::new(self.store.clone());

        let resolved_id = match self.resolve_node_id(&repo, &query.entity_id) {
            Ok(id) => id,
            Err(_) => {
                return Ok(EngineeringEvidence::not_found(
                    &query.entity_id,
                    &query.project_id,
                ))
            }
        };

        let node_id = NodeId::from(resolved_id.as_str());

        let node = match repo.get_node(&node_id)? {
            Some(n) => n,
            None => {
                return Ok(EngineeringEvidence::not_found(
                    &query.entity_id,
                    &query.project_id,
                ))
            }
        };

        let outgoing_edges = repo.get_edges_from(&node_id).unwrap_or_default();
        let incoming_edges = repo.get_edges_to(&node_id).unwrap_or_default();

        // ── 1. Classify edges into semantic buckets ──────────────
        let mut folders: Vec<EntityRef> = Vec::new();
        let mut parent_module: Option<EntityRef> = None;
        let mut dependencies: Vec<DependencyRef> = Vec::new();
        let mut dependents: Vec<DependencyRef> = Vec::new();
        let mut owner_ids: Vec<String> = Vec::new();

        // Incoming edges
        for edge in &incoming_edges {
            let neighbor = match repo.get_node(&edge.from_node_id).ok().flatten() {
                Some(n) => n,
                None => continue,
            };
            let etype = edge.edge_type.as_str();
            let ntype = neighbor.node_type.as_str();

            match etype {
                "contains" => {
                    let ent = EntityRef {
                        id: edge.from_node_id.as_str().to_string(),
                        label: neighbor.label.clone(),
                        node_type: ntype.to_string(),
                        file_path: neighbor.file_path.clone(),
                    };
                    if ntype == "module" && parent_module.is_none() {
                        parent_module = Some(ent);
                    } else {
                        folders.push(ent);
                    }
                }
                "touches" | "contributed_to" => {
                    // Handled by dedicated commit/contributor traversal below
                }
                "owns" => {
                    if !owner_ids.contains(&edge.from_node_id.as_str().to_string()) {
                        owner_ids.push(edge.from_node_id.as_str().to_string());
                    }
                }
                _ => {
                    dependents.push(DependencyRef {
                        id: edge.from_node_id.as_str().to_string(),
                        label: neighbor.label.clone(),
                        node_type: ntype.to_string(),
                        file_path: neighbor.file_path.clone(),
                        relationship: etype.to_string(),
                        is_test: Self::is_test_entity(&neighbor.label, &neighbor.file_path, ntype),
                    });
                }
            }
        }

        // Outgoing edges
        for edge in &outgoing_edges {
            let neighbor = match repo.get_node(&edge.to_node_id).ok().flatten() {
                Some(n) => n,
                None => continue,
            };
            let etype = edge.edge_type.as_str();
            let ntype = neighbor.node_type.as_str();

            match etype {
                "contained_in" => {
                    let nid = edge.to_node_id.as_str().to_string();
                    // Deduplicate — may already have from incoming "contains"
                    if !folders.iter().any(|f| f.id == nid) {
                        folders.push(EntityRef {
                            id: nid,
                            label: neighbor.label.clone(),
                            node_type: ntype.to_string(),
                            file_path: neighbor.file_path.clone(),
                        });
                    }
                }
                "owned_by" | "authored_by" => {
                    if etype == "owned_by"
                        && !owner_ids.contains(&edge.to_node_id.as_str().to_string())
                    {
                        owner_ids.push(edge.to_node_id.as_str().to_string());
                    }
                    // authored_by is handled in commit traversal
                }
                _ => {
                    dependencies.push(DependencyRef {
                        id: edge.to_node_id.as_str().to_string(),
                        label: neighbor.label.clone(),
                        node_type: ntype.to_string(),
                        file_path: neighbor.file_path.clone(),
                        relationship: etype.to_string(),
                        is_test: Self::is_test_entity(&neighbor.label, &neighbor.file_path, ntype),
                    });
                }
            }
        }

        // ── 2. Resolve owner labels ──────────────────────────────
        let owners: Vec<String> = owner_ids
            .iter()
            .filter_map(|oid| repo.get_node(&NodeId::from(oid.as_str())).ok().flatten())
            .map(|n| n.label)
            .collect();

        // ── 3. Multi-hop: git commits + contributors ─────────────
        let mut git_commits = Vec::new();
        let mut author_counts: HashMap<String, usize> = HashMap::new();

        for edge in incoming_edges
            .iter()
            .filter(|e| e.edge_type.as_str() == "touches")
        {
            let commit_node = match repo.get_node(&edge.from_node_id).ok().flatten() {
                Some(n) => n,
                None => continue,
            };

            let hash = commit_node
                .properties
                .get("hash")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let message = commit_node
                .properties
                .get("subject")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let ts = commit_node
                .properties
                .get("timestamp")
                .and_then(|v| v.as_i64());

            // Hop 2: commit → authored_by → person
            let mut author_name = None;
            if let Ok(commit_edges) = repo.get_edges_from(&edge.from_node_id) {
                for ce in commit_edges
                    .iter()
                    .filter(|e| e.edge_type.as_str() == "authored_by")
                {
                    if let Some(person) = repo.get_node(&ce.to_node_id).ok().flatten() {
                        *author_counts.entry(person.label.clone()).or_insert(0) += 1;
                        author_name = Some(person.label.clone());
                    }
                }
            }

            git_commits.push(GitEvidence {
                hash,
                message: message.lines().next().unwrap_or("").to_string(),
                date: ts.map(format_timestamp).unwrap_or_default(),
                author: author_name.unwrap_or_default(),
            });
        }

        // Sort most-recent-first, cap at 20
        git_commits.sort_by(|a, b| b.date.cmp(&a.date));
        git_commits.truncate(20);

        // Build contributor list sorted by commit count descending
        let mut contributors: Vec<ContributorRef> = author_counts
            .into_iter()
            .map(|(name, count)| ContributorRef {
                name,
                commit_count: count,
                is_primary: false,
            })
            .collect();
        contributors.sort_by(|a, b| b.commit_count.cmp(&a.commit_count));
        if let Some(first) = contributors.first_mut() {
            first.is_primary = true;
        }

        // ── 4. Metrics ──────────────────────────────────────────
        let loc = node
            .properties
            .get("loc")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        let complexity = node
            .properties
            .get("complexity")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32);

        let metrics = EntityMetrics {
            lines_of_code: loc,
            complexity,
            test_coverage: None,
            dependency_count: dependencies.len(),
            dependent_count: dependents.len(),
        };

        let timestamps = Timestamps {
            created_at: Some(format_timestamp(node.created_at)),
            updated_at: Some(format_timestamp(node.updated_at)),
            last_committed: None,
        };

        Ok(EngineeringEvidence {
            entity_id: node.id.as_str().to_string(),
            entity_type: node.node_type.as_str().to_string(),
            entity_label: node.label.clone(),
            file_path: node.file_path.clone(),
            project_id: query.project_id.clone(),
            folders,
            parent_module,
            dependencies,
            dependents,
            contributors,
            owners,
            commits: git_commits,
            requirements: Vec::new(),
            decisions: Vec::new(),
            symbols: Vec::new(),
            references: Vec::new(),
            tests: Vec::new(),
            documentation: Vec::new(),
            metrics: Some(metrics),
            timestamps: Some(timestamps),
            risk: None,
        })
    }
}

fn format_timestamp(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_else(|| ts.to_string())
}
