use std::path::{Path, PathBuf};
use std::fs;

pub struct TypeScriptDependencyExtractor;

impl TypeScriptDependencyExtractor {
    pub fn extract_dependencies(package_json_path: &Path) -> Vec<(String, String)> {
        let mut deps = Vec::new();
        if let Ok(content) = fs::read_to_string(package_json_path) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(dependencies) = parsed.get("dependencies").and_then(|v| v.as_object()) {
                    for (dep_name, _) in dependencies {
                        deps.push((package_json_path.to_string_lossy().to_string(), dep_name.clone()));
                    }
                }
                if let Some(dev_dependencies) = parsed.get("devDependencies").and_then(|v| v.as_object()) {
                    for (dep_name, _) in dev_dependencies {
                        deps.push((package_json_path.to_string_lossy().to_string(), dep_name.clone()));
                    }
                }
            }
        }
        deps
    }
}
