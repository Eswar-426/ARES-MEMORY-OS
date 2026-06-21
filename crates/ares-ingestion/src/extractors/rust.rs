use std::path::{Path, PathBuf};
use std::fs;
use ares_knowledge_graph::models::{KnowledgeEdge, EdgeType};

pub struct RustDependencyExtractor;

impl RustDependencyExtractor {
    pub fn extract_dependencies(cargo_toml_path: &Path) -> Vec<(String, String)> {
        let mut deps = Vec::new();
        if let Ok(content) = fs::read_to_string(cargo_toml_path) {
            if let Ok(parsed) = content.parse::<toml::Value>() {
                if let Some(dependencies) = parsed.get("dependencies").and_then(|v| v.as_table()) {
                    for (dep_name, _) in dependencies {
                        deps.push((cargo_toml_path.to_string_lossy().to_string(), dep_name.clone()));
                    }
                }
                if let Some(dev_dependencies) = parsed.get("dev-dependencies").and_then(|v| v.as_table()) {
                    for (dep_name, _) in dev_dependencies {
                        deps.push((cargo_toml_path.to_string_lossy().to_string(), dep_name.clone()));
                    }
                }
            }
        }
        deps
    }
}
