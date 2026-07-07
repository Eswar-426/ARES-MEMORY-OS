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
        // %H: hash, %an: author name, %ae: author email, %at: author time, %s: subject, %B: body
        let format = "--format=[COMMIT]%x00%H%x00%an%x00%ae%x00%at%x00%s%x00%B%x00[FILES]";

        let mut cmd = Command::new("git");
        cmd.current_dir(project_path).args([
            "log",
            "-m",
            "--first-parent",
            &format!("-{}", depth),
            format,
            "--name-status",
        ]);

        use std::process::Stdio;
        use std::time::Duration;
        use wait_timeout::ChildExt;

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to execute git log: {}", e))?;

        let timeout = Duration::from_secs(60); // Log can take longer than blame
        let status = match child.wait_timeout(timeout) {
            Ok(Some(status)) => status,
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("git log timed out after 60 seconds".to_string());
            }
            Err(e) => {
                return Err(format!("wait_timeout error: {}", e));
            }
        };

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        if let Some(mut out) = child.stdout.take() {
            use std::io::Read;
            let _ = out.read_to_end(&mut stdout);
        }
        if let Some(mut err) = child.stderr.take() {
            use std::io::Read;
            let _ = err.read_to_end(&mut stderr);
        }

        if !status.success() {
            let stderr_str = String::from_utf8_lossy(&stderr);
            if stderr_str.contains("not a git repository")
                || stderr_str.contains("does not have any commits")
            {
                return Ok((vec![], vec![])); // Quietly return empty for non-git repos
            }
            return Err(format!("git log failed: {}", stderr_str));
        }

        let output_str = String::from_utf8_lossy(&stdout);

        let mut seen_persons = HashSet::new();
        let commits = output_str.split("[COMMIT]\0").filter(|s| !s.is_empty());

        for commit_block in commits {
            let mut parts = commit_block.splitn(2, "\0[FILES]\n").collect::<Vec<_>>();
            let metadata_part = parts[0];
            let files_part = if parts.len() > 1 { parts[1] } else { "" };

            let meta_parts: Vec<&str> = metadata_part.split('\0').collect();
            if meta_parts.len() < 6 {
                continue;
            }

            let hash = meta_parts[0];
            let author_name = meta_parts[1];
            let author_email = meta_parts[2];
            let timestamp: i64 = meta_parts[3].parse().unwrap_or(captured_at);
            let subject = meta_parts[4];
            let body = meta_parts[5];

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
                "body": body,
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

            // 4. Extract Decisions
            let mut decision_text = String::new();
            let mut reason_text = String::new();
            let mut tradeoff_text = String::new();

            for line in body.lines() {
                if let Some(rest) = line.strip_prefix("Decision:") {
                    decision_text.push_str(rest.trim());
                    decision_text.push('\n');
                } else if let Some(rest) = line.strip_prefix("Reason:") {
                    reason_text.push_str(rest.trim());
                    reason_text.push('\n');
                } else if let Some(rest) = line.strip_prefix("Tradeoff:") {
                    tradeoff_text.push_str(rest.trim());
                    tradeoff_text.push('\n');
                }
            }

            if !decision_text.is_empty() {
                let dec_id = NodeId::from(format!("decision:{}", hash));
                let mut d_props = serde_json::json!({
                    "decision": decision_text.trim(),
                    "reason": reason_text.trim(),
                    "tradeoff": tradeoff_text.trim(),
                });
                if let Some(p) = d_props.as_object_mut() {
                    p.insert("provenance".to_string(), prov_val.clone());
                }

                nodes.push(GraphNode {
                    id: dec_id.clone(),
                    project_id: project_id.clone(),
                    node_type: NodeType::Decision,
                    label: decision_text.chars().take(100).collect(),
                    properties: d_props,
                    file_path: None,
                    created_at: timestamp,
                    updated_at: timestamp,
                    deleted_at: None,
                });

                edges.push(GraphEdge {
                    id: format!("{}-contains-{}", commit_id.as_str(), dec_id.as_str()),
                    project_id: project_id.clone(),
                    from_node_id: commit_id.clone(),
                    to_node_id: dec_id,
                    edge_type: EdgeType::Contains,
                    weight: 1.0,
                    confidence: 1.0,
                    source: "git_commits".to_string(),
                    valid_from: timestamp,
                    valid_until: None,
                    created_at: captured_at,
                });
            }

            // 5. Process Touched Files
            for line in files_part.lines() {
                if line.is_empty() {
                    continue;
                }
                let f_parts: Vec<&str> = line.split('\t').collect();
                if f_parts.len() >= 2 {
                    let status = f_parts[0];
                    let file_path = if status.starts_with('R') && f_parts.len() >= 3 {
                        f_parts[2] // new name on rename
                    } else {
                        f_parts[1]
                    };

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
                        valid_from: timestamp,
                        valid_until: None,
                        created_at: captured_at,
                    });
                }
            }
        }

        Ok((nodes, edges))
    }
}
