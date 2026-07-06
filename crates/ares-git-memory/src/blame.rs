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
        let mut files: Vec<&str> = files_str.lines().filter(|l| !l.is_empty()).collect();

        // 2. Filter out known binary, generated, and asset files
        let blocklist = [
            // Images & Media
            ".png", ".jpg", ".jpeg", ".gif", ".bmp", ".tiff", ".ico", ".svg", ".webp",
            ".mp3", ".mp4", ".wav", ".avi", ".mov", ".mkv", ".webm",
            // Fonts
            ".woff", ".woff2", ".eot", ".ttf", ".otf",
            // Compiled & Archives
            ".dll", ".exe", ".so", ".dylib", ".class", ".jar", ".war", ".pyc", ".pyo", ".pyd",
            ".zip", ".tar", ".gz", ".tgz", ".rar", ".7z", ".bz2", ".xz", ".whl", ".egg", ".gem",
            // Object files and debug symbols
            ".o", ".obj", ".a", ".lib", ".pdb", ".out",
            // Data & Binary formats
            ".parquet", ".avro", ".sqlite", ".sqlite3", ".db", ".bin", ".dat", ".wasm", ".pdf",
            // Translations & Localizations
            ".mo", ".po", ".pot",
            // ML Weights & Models
            ".npz", ".npy", ".h5", ".pb", ".pt", ".pth", ".onnx", ".safetensors",
            // Minified JS/CSS and map files
            ".min.js", ".min.css", ".map",
            // Test snapshots (huge, generated)
            ".snap",
            // Certificates & Keys
            ".pem", ".crt", ".key", ".p12", ".pfx",
            // Logs and Locks
            ".log", "package-lock.json", "yarn.lock", "pnpm-lock.yaml", "cargo.lock", "poetry.lock", "gemfile.lock", "composer.lock",
        ];

        let dir_blocklist = [
            "venv/", ".venv/", "env/", ".env/", "node_modules/", "bower_components/",
            "__pycache__/", ".pytest_cache/", ".mypy_cache/", ".tox/",
            "target/", "build/", "dist/", "out/", "bin/", "obj/",
            ".git/", ".svn/", ".hg/", ".idea/", ".vscode/",
            "vendor/", ".next/", ".nuxt/",
        ];

        files.retain(|f| {
            let lower = f.to_lowercase().replace('\\', "/");
            let has_blocked_ext = blocklist.iter().any(|&ext| lower.ends_with(ext));
            let has_blocked_dir = dir_blocklist.iter().any(|&dir| lower.contains(dir));
            !has_blocked_ext && !has_blocked_dir
        });

        let mut seen_persons = HashSet::new();

        println!("Running git blame on {} files...", files.len());

        struct CreationInfo {
            hash: String,
            author: String,
            reason: String,
            timestamp: i64,
        }

        let mut creation_map: HashMap<String, CreationInfo> = HashMap::new();
        println!("Pre-computing creation commits for files...");
        let mut global_creation_cmd = Command::new("git");
        global_creation_cmd
            .current_dir(project_path)
            .args(["log", "--name-status", "--pretty=format:COMMIT|%H|%an|%s|%at", "--reverse"]);

        if let Ok(output) = global_creation_cmd.output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut current_hash = String::new();
            let mut current_author = String::new();
            let mut current_subject = String::new();
            let mut current_timestamp = 0;

            for line in output_str.lines() {
                if line.starts_with("COMMIT|") {
                    let parts: Vec<&str> = line.splitn(5, '|').collect();
                    if parts.len() == 5 {
                        current_hash = parts[1].to_string();
                        current_author = parts[2].to_string();
                        current_subject = parts[3].to_string();
                        current_timestamp = parts[4].parse().unwrap_or(captured_at);
                    }
                } else if line.starts_with(|c: char| c.is_ascii_uppercase()) && line.contains('\t') {
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() >= 2 {
                        let file_path = parts.last().unwrap().to_string();
                        creation_map.entry(file_path).or_insert(CreationInfo {
                            hash: current_hash.clone(),
                            author: current_author.clone(),
                            reason: current_subject.clone(),
                            timestamp: current_timestamp,
                        });
                    }
                }
            }
        }

        use rayon::prelude::*;

        let results: Vec<(Vec<GraphNode>, Vec<GraphEdge>)> = files
            .par_iter()
            .filter_map(|&file| {
                let mut local_nodes = Vec::new();
                let mut local_edges = Vec::new();

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

                            let prov_val =
                                serde_json::to_value(&prov).unwrap_or(serde_json::json!({}));

                            let mut props = serde_json::json!({
                                "name": name,
                                "email": email,
                            });
                            if let Some(p) = props.as_object_mut() {
                                p.insert("provenance".to_string(), prov_val.clone());
                            }

                            local_nodes.push(GraphNode {
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

                            local_edges.push(GraphEdge {
                                id: format!(
                                    "{}-contributedto-{}",
                                    person_id.as_str(),
                                    file_node_id
                                ),
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
                        }
                    }
                }

                // --- NEW: Introduction / Creation extraction (O(1) lookup) ---
                if let Some(info) = creation_map.get(file) {
                    let file_node_id = ares_core::canonicalize_node_id(file);
                    
                    local_nodes.push(GraphNode {
                        id: NodeId::from(file_node_id.as_str()),
                        project_id: project_id.clone(),
                        node_type: NodeType::File,
                        label: String::new(),
                        properties: serde_json::json!({
                            "introduced_at": info.timestamp,
                            "introduced_by": info.author,
                            "introduction_reason": info.reason,
                            "introduction_hash": info.hash,
                        }),
                        file_path: Some(file.to_string()),
                        created_at: captured_at,
                        updated_at: captured_at,
                        deleted_at: None,
                    });
                }
                // -------------------------------------------------------------

                Some((local_nodes, local_edges))
            })
            .collect();

        for (mut local_nodes, mut local_edges) in results {
            for node in local_nodes.drain(..) {
                if !seen_persons.contains(&node.id) {
                    seen_persons.insert(node.id.clone());
                    nodes.push(node);
                }
            }
            edges.append(&mut local_edges);
        }

        Ok((nodes, edges))
    }
}
