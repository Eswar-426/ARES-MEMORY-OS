use crate::models::{CaptureMethod, SourceProvenance};
use ares_core::{EdgeType, GraphEdge, GraphNode, NodeId, NodeType, ProjectId};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

pub struct BlameExtractor;

impl BlameExtractor {
    pub fn extract(
        project_path: &Path,
        project_id: &ProjectId,
        captured_at: i64,
    ) -> Result<(Vec<GraphNode>, Vec<GraphEdge>), String> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // 1. Get all tracked files
        let mut ls_cmd = Command::new("git");
        ls_cmd.current_dir(project_path).args(["ls-files"]);

        let ls_output = ls_cmd
            .output()
            .map_err(|e| format!("git ls-files failed: {}", e))?;
        if !ls_output.status.success() {
            return Ok((vec![], vec![]));
        }

        let files_str = String::from_utf8_lossy(&ls_output.stdout);
        let files: Vec<&str> = files_str.lines().filter(|l| !l.is_empty()).collect();

        let mut seen_persons = HashSet::new();

        println!("Running git blame on {} files...", files.len());

        for file in files {
            // Run blame on each file
            let mut blame_cmd = Command::new("git");
            blame_cmd
                .current_dir(project_path)
                .args(["blame", "--line-porcelain", file]);

            if let Ok(output) = blame_cmd.output() {
                if output.status.success() {
                    let blame_str = String::from_utf8_lossy(&output.stdout);

                    let mut author_lines: HashMap<String, (String, usize)> = HashMap::new(); // email -> (name, count)
                    let mut total_lines = 0;

                    let mut current_author = String::new();
                    let mut current_email = String::new();

                    for line in blame_str.lines() {
                        if line.starts_with("author ") {
                            current_author = line[7..].to_string();
                        } else if line.starts_with("author-mail ") {
                            current_email = line[12..]
                                .trim_matches(|c| c == '<' || c == '>')
                                .to_string();
                        } else if line.starts_with('\t') {
                            // This is the actual line content
                            let entry = author_lines
                                .entry(current_email.clone())
                                .or_insert((current_author.clone(), 0));
                            entry.1 += 1;
                            total_lines += 1;
                        }
                    }

                    let file_node_id = ares_core::canonicalize_node_id(file);

                    // Generate Person nodes and ContributedTo edges
                    for (email, (name, count)) in author_lines {
                        let person_id = NodeId::from(format!("person:{}", email));

                        let prov = SourceProvenance {
                            source_system: "git_blame".to_string(),
                            source_id: file.to_string(),
                            capture_method: CaptureMethod::Heuristic,
                            captured_at,
                            confidence: CaptureMethod::Heuristic.base_confidence(), // 0.4
                        };

                        let prov_val = serde_json::to_value(&prov).unwrap_or(serde_json::json!({}));

                        if !seen_persons.contains(&person_id) {
                            seen_persons.insert(person_id.clone());

                            let mut props = serde_json::json!({
                                "name": name,
                                "email": email,
                            });
                            if let Some(p) = props.as_object_mut() {
                                p.insert("provenance".to_string(), prov_val.clone());
                            }

                            nodes.push(GraphNode {
                                id: person_id.clone(),
                                project_id: project_id.clone(),
                                node_type: NodeType::Person,
                                label: name.clone(),
                                properties: props,
                                file_path: None,
                                created_at: captured_at,
                                updated_at: captured_at,
                                deleted_at: None,
                            });
                        }

                        let percentage = if total_lines > 0 {
                            (count as f64) / (total_lines as f64)
                        } else {
                            0.0
                        };

                        // Compute edge confidence based on percentage
                        let edge_confidence = if percentage > 0.5 {
                            0.7 // High contribution
                        } else if percentage > 0.2 {
                            0.5 // Medium
                        } else {
                            0.4 // Minor
                        };

                        edges.push(GraphEdge {
                            id: format!("{}-contributedto-{}", person_id.as_str(), file_node_id),
                            project_id: project_id.clone(),
                            from_node_id: person_id.clone(),
                            to_node_id: NodeId::from(file_node_id.as_str()),
                            edge_type: EdgeType::ContributedTo,
                            weight: percentage as f32,
                            confidence: edge_confidence as f32,
                            source: "git_blame".to_string(),
                            valid_from: captured_at,
                            valid_until: None,
                            created_at: captured_at,
                        });

                        // If they own > 80% of lines, we might also infer `Maintains`,
                        // but per Architecture, Maintains is explicit or based on activity.
                        // For now, we capture ContributedTo as the base fact.
                    }
                }
            }
        }

        Ok((nodes, edges))
    }
}
