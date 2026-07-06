use std::fs;
use std::path::Path;

pub struct OwnershipExtractor;

impl OwnershipExtractor {
    pub fn extract_ownership(root: &Path) -> Vec<(String, String)> {
        let mut ownership = Vec::new();

        let codeowners_path = root.join("CODEOWNERS");
        if let Ok(content) = fs::read_to_string(codeowners_path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let pattern = parts[0];
                    let owner = parts[1].trim_matches(|c| c == '"' || c == '\'');
                    ownership.push((pattern.to_string(), owner.to_string()));
                }
            }
        }

        // Also check .github/CODEOWNERS
        let github_codeowners = root.join(".github").join("CODEOWNERS");
        if let Ok(content) = fs::read_to_string(github_codeowners) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let pattern = parts[0];
                    let owner = parts[1].trim_matches(|c| c == '"' || c == '\'');
                    ownership.push((pattern.to_string(), owner.to_string()));
                }
            }
        }
        // Also check ownership.md
        let ownership_md = root.join("ownership.md");
        if let Ok(content) = fs::read_to_string(ownership_md) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let pattern = parts[0];
                    let owner = parts[1].trim_matches(|c| c == '"' || c == '\'');
                    ownership.push((pattern.to_string(), owner.to_string()));
                }
            }
        }

        ownership
    }
}
