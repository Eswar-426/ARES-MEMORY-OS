use crate::models::{CaptureMethod, SourceProvenance};
use ares_core::{EdgeType, GraphEdge, GraphNode, NodeId, NodeType, ProjectId};
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

pub struct CommitExtractor;

impl CommitExtractor {
    pub fn extract(
        project_path: &Path,
        project_id: &ProjectId,
        depth: usize,
        captured_at: i64,
    ) -> Result<(Vec<GraphNode>, Vec<GraphEdge>), String> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Use null bytes as delimiters to avoid parsing issues with messages
        // %H: hash, %an: author name, %ae: author email, %at: author time, %B: raw body (message)
        // We'll use a separator like |||ARES||| for body since %B can contain nulls/newlines
        let format = "--format=[COMMIT]%x00%H%x00%an%x00%ae%x00%at%x00%s";

        let mut cmd = Command::new("git");
        cmd.current_dir(project_path).args([
            "log",
            &format!("-{}", depth),
            format,
            "--name-status",
        ]);

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute git log: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("not a git repository")
                || stderr.contains("does not have any commits")
            {
                return Ok((vec![], vec![])); // Quietly return empty for non-git repos
            }
            return Err(format!("git log failed: {}", stderr));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);

        let mut current_commit_id = None;
        let mut current_author_id = None;
        let mut seen_persons = HashSet::new();

        for line in output_str.lines() {
            if line.is_empty() {
                continue;
            }

            if line.starts_with("[COMMIT]\0") {
                let parts: Vec<&str> = line.split('\0').collect();
                if parts.len() < 6 {
                    continue;
                }

                let hash = parts[1];
                let author_name = parts[2];
                let author_email = parts[3];
                let timestamp: i64 = parts[4].parse().unwrap_or(captured_at);
                let subject = parts[5];

                let commit_id = NodeId::from(format!("commit:{}", hash));
                let person_id = NodeId::from(format!("person:{}", author_email));

                let prov = SourceProvenance {
                    source_system: "git_log".to_string(),
                    source_id: hash.to_string(),
                    capture_method: CaptureMethod::Repository,
                    captured_at,
                    confidence: CaptureMethod::Repository.base_confidence(),
                };

                let prov_val = serde_json::to_value(&prov).unwrap_or(serde_json::json!({}));

                // 1. Create Commit Node
                let mut props = serde_json::json!({
                    "hash": hash,
                    "author": author_name,
                    "email": author_email,
                    "subject": subject,
                });
                if let Some(p) = props.as_object_mut() {
                    p.insert("provenance".to_string(), prov_val.clone());
                }

                nodes.push(GraphNode {
                    id: commit_id.clone(),
                    project_id: project_id.clone(),
                    node_type: NodeType::Commit,
                    label: subject.chars().take(100).collect(),
                    properties: props,
                    file_path: None,
                    created_at: timestamp,
                    updated_at: timestamp,
                    deleted_at: None,
                });

                // 2. Create Person Node (if not seen)
                if !seen_persons.contains(&person_id) {
                    seen_persons.insert(person_id.clone());

                    let mut person_props = serde_json::json!({
                        "name": author_name,
                        "email": author_email,
                    });
                    if let Some(p) = person_props.as_object_mut() {
                        p.insert("provenance".to_string(), prov_val.clone());
                    }

                    nodes.push(GraphNode {
                        id: person_id.clone(),
                        project_id: project_id.clone(),
                        node_type: NodeType::Person,
                        label: author_name.to_string(),
                        properties: person_props,
                        file_path: None,
                        created_at: timestamp,
                        updated_at: timestamp,
                        deleted_at: None,
                    });
                }

                // 3. Create AuthoredBy Edge (Commit -> Person)
                let _edge_prov = SourceProvenance {
                    source_system: "git_log".to_string(),
                    source_id: hash.to_string(),
                    capture_method: CaptureMethod::Explicit, // Authorship is explicit fact
                    captured_at,
                    confidence: 1.0,
                };

                edges.push(GraphEdge {
                    id: format!("{}-authoredby-{}", commit_id.as_str(), person_id.as_str()),
                    project_id: project_id.clone(),
                    from_node_id: commit_id.clone(),
                    to_node_id: person_id.clone(),
                    edge_type: EdgeType::AuthoredBy,
                    weight: 1.0,
                    confidence: 1.0,
                    source: "git_commits".to_string(),
                    valid_from: timestamp,
                    valid_until: None,
                    created_at: captured_at,
                });

                current_commit_id = Some((commit_id, timestamp, hash.to_string()));
                current_author_id = Some(person_id);
            } else if let Some((commit_id, timestamp, _hash)) = &current_commit_id {
                // Name-status lines: M\tfile.rs or R100\told\tnew
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 2 {
                    let status = parts[0];
                    let file_path = if status.starts_with('R') && parts.len() >= 3 {
                        parts[2] // Get new name on rename
                    } else {
                        parts[1]
                    };

                    // We link Commit -> File
                    let file_node_id = ares_core::canonicalize_node_id(file_path);

                    edges.push(GraphEdge {
                        id: format!("{}-touches-{}", commit_id.as_str(), file_node_id),
                        project_id: project_id.clone(),
                        from_node_id: commit_id.clone(),
                        to_node_id: NodeId::from(file_node_id.as_str()),
                        edge_type: EdgeType::Touches,
                        weight: 1.0,
                        confidence: 0.8,
                        source: "git_commits".to_string(),
                        valid_from: *timestamp,
                        valid_until: None,
                        created_at: captured_at,
                    });
                }
            }
        }

        Ok((nodes, edges))
    }
}
