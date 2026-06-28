use crate::engines::overview::models::RepositoryOverview;
use ares_store::Store;
use std::path::Path;
use std::process::Command;

pub async fn collect(store: &Store, project_path: &str) -> RepositoryOverview {
    let name = Path::new(project_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let root_path = project_path.to_string();
    
    // Get git info
    let branch = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .current_dir(project_path)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "main".to_string());

    let commit = Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .current_dir(project_path)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let stats = store.overview_repository_stats().unwrap_or_else(|_| ares_store::overview::RepositoryStats {
        files: 0,
        functions: 0,
        directories: 0,
        modules: 0,
    });

    let mut language = "Unknown".to_string();
    if let Ok(conn) = store.get_conn() {
        if let Ok(props_str) = conn.query_row(
            &format!("SELECT properties FROM graph_nodes WHERE node_type = '{}' LIMIT 1", ares_core::NodeType::Project.as_str()),
            [],
            |row| row.get::<_, String>(0)
        ) {
            if let Ok(props) = serde_json::from_str::<serde_json::Value>(&props_str) {
                if let Some(lang) = props.get("language").and_then(|v| v.as_str()) {
                    language = lang.to_string();
                }
            }
        }
    }

    RepositoryOverview {
        name,
        root_path,
        language,
        branch,
        commit,
        files: stats.files,
        functions: stats.functions,
        directories: stats.directories,
        modules: stats.modules,
        indexed: true,
        last_ingest: "Just now".to_string(),
        is_dirty: false,
    }
}
