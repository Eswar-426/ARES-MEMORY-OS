use crate::models::{CaptureMethod, SourceProvenance};
use ares_core::{EdgeType, GraphEdge, GraphNode, NodeId, NodeType, ProjectId};
use std::path::Path;
use std::process::Command;

pub struct BranchExtractor;

impl BranchExtractor {
    pub fn extract(
        project_path: &Path,
        project_id: &ProjectId,
        captured_at: i64,
    ) -> Result<(Vec<GraphNode>, Vec<GraphEdge>), String> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Get all local branches and their tip commits
        let mut cmd = Command::new("git");
        cmd.current_dir(project_path).args([
            "for-each-ref",
            "--format=%(refname:short)%x00%(objectname)%x00%(committerdate:unix)",
            "refs/heads",
            "refs/remotes",
        ]);

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute git for-each-ref: {}", e))?;

        if !output.status.success() {
            return Ok((vec![], vec![])); // Quietly return empty for non-git repos
        }

        let output_str = String::from_utf8_lossy(&output.stdout);

        for line in output_str.lines() {
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('\0').collect();
            if parts.len() < 3 {
                continue;
            }

            let branch_name = parts[0];
            let commit_hash = parts[1];
            let date_str = parts[2];

            // For remotes, the name might be origin/main
            // We'll just use the short name
            let timestamp: i64 = date_str.parse().unwrap_or(captured_at);

            let branch_id = NodeId::from(format!("branch:{}", branch_name));
            let commit_id = NodeId::from(format!("commit:{}", commit_hash));

            let prov = SourceProvenance {
                source_system: "git_branch".to_string(),
                source_id: branch_name.to_string(),
                capture_method: CaptureMethod::Repository,
                captured_at,
                confidence: CaptureMethod::Repository.base_confidence(),
            };

            let prov_val = serde_json::to_value(&prov).unwrap_or(serde_json::json!({}));

            // 1. Create Branch Node
            let mut props = serde_json::json!({
                "branch": branch_name,
                "commit": commit_hash,
            });
            if let Some(p) = props.as_object_mut() {
                p.insert("provenance".to_string(), prov_val);
            }

            nodes.push(GraphNode {
                id: branch_id.clone(),
                project_id: project_id.clone(),
                node_type: NodeType::Branch,
                label: branch_name.to_string(),
                properties: props,
                file_path: None,
                created_at: timestamp,
                updated_at: timestamp,
                deleted_at: None,
            });

            // 2. Create Contains Edge (Branch -> Commit)
            edges.push(GraphEdge {
                id: format!("{}-contains-{}", branch_id.as_str(), commit_id.as_str()),
                project_id: project_id.clone(),
                from_node_id: branch_id.clone(),
                to_node_id: commit_id.clone(),
                edge_type: EdgeType::Contains,
                weight: 1.0,
                confidence: 0.8, // Repository tier confidence
                source: "git_branch".to_string(),
                valid_from: timestamp,
                valid_until: None,
                created_at: captured_at,
            });
        }

        Ok((nodes, edges))
    }
}
