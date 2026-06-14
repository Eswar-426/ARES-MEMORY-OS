//! Analyzers — extract architecture, language, and dependency information from project files.

use crate::types::*;
use std::collections::HashMap;
use std::path::Path;
use tracing::debug;
use walkdir::WalkDir;

// ─────────────────────────────────────────────────────────────────
// Architecture Analyzer
// ─────────────────────────────────────────────────────────────────

pub struct ArchitectureAnalyzer;

impl ArchitectureAnalyzer {
    /// Analyze the project root to determine architecture style and components.
    pub fn analyze(root_path: &Path) -> ArchitectureProfile {
        let mut components = Vec::new();
        let mut patterns = Vec::new();
        let mut entry_points = Vec::new();

        // Detect architecture style from directory structure
        let style =
            Self::detect_style(root_path, &mut components, &mut patterns, &mut entry_points);

        debug!(style = ?style, components = components.len(), "Architecture analysis complete");

        ArchitectureProfile {
            style,
            components,
            patterns,
            entry_points,
        }
    }

    fn detect_style(
        root: &Path,
        components: &mut Vec<ComponentInfo>,
        patterns: &mut Vec<String>,
        entry_points: &mut Vec<String>,
    ) -> ArchitectureStyle {
        let mut has_crates = false;
        let mut has_packages = false;
        let mut has_services = false;
        let mut has_apps = false;
        let mut has_serverless_config = false;

        for entry in WalkDir::new(root)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let rel = path.strip_prefix(root).unwrap_or(path);
            let rel_str = rel.to_string_lossy().to_string();

            if entry.depth() == 1 && path.is_dir() {
                match name.as_str() {
                    "crates" => {
                        has_crates = true;
                        Self::scan_subdir(path, "crate", components);
                    }
                    "packages" => {
                        has_packages = true;
                        Self::scan_subdir(path, "package", components);
                    }
                    "services" => {
                        has_services = true;
                        Self::scan_subdir(path, "service", components);
                    }
                    "apps" => {
                        has_apps = true;
                        Self::scan_subdir(path, "app", components);
                    }
                    "extensions" => {
                        Self::scan_subdir(path, "extension", components);
                    }
                    _ => {}
                }
            }

            // Detect patterns from config files
            if path.is_file() {
                match name.as_str() {
                    "serverless.yml" | "serverless.yaml" | "sam.yaml" => {
                        has_serverless_config = true
                    }
                    "docker-compose.yml" | "docker-compose.yaml" => {
                        patterns.push("Docker Compose".into())
                    }
                    "Dockerfile" => patterns.push("Containerized".into()),
                    "turbo.json" => patterns.push("Turborepo monorepo".into()),
                    "pnpm-workspace.yaml" => patterns.push("pnpm workspace".into()),
                    "Cargo.toml" if entry.depth() == 0 => {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            if content.contains("[workspace]") {
                                patterns.push("Cargo workspace".into());
                            }
                        }
                    }
                    _ => {}
                }

                // Detect entry points
                if name == "main.rs"
                    || name == "main.ts"
                    || name == "main.py"
                    || name == "main.go"
                    || name == "index.ts"
                    || name == "index.js"
                    || name == "app.py"
                {
                    entry_points.push(rel_str);
                }
            }
        }

        // Deduplicate patterns
        patterns.sort();
        patterns.dedup();

        if has_serverless_config {
            ArchitectureStyle::Serverless
        } else if has_services {
            ArchitectureStyle::Microservices
        } else if has_crates || has_packages || has_apps {
            ArchitectureStyle::Modular
        } else {
            ArchitectureStyle::Monolith
        }
    }

    fn scan_subdir(dir: &Path, comp_type: &str, components: &mut Vec<ComponentInfo>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    components.push(ComponentInfo {
                        name: name.clone(),
                        path: path.to_string_lossy().to_string(),
                        component_type: comp_type.into(),
                        description: format!("{comp_type}: {name}"),
                    });
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────
// Language Analyzer
// ─────────────────────────────────────────────────────────────────

pub struct LanguageAnalyzer;

impl LanguageAnalyzer {
    /// Count files and lines by language extension.
    pub fn analyze(root_path: &Path) -> Vec<LanguageProfile> {
        let mut counts: HashMap<String, (u32, u64)> = HashMap::new();

        for entry in WalkDir::new(root_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();

            // Skip hidden, target, node_modules, .git
            let path_str = path.to_string_lossy();
            if path_str.contains("target/")
                || path_str.contains("node_modules/")
                || path_str.contains(".git/")
                || path_str.contains("target\\")
                || path_str.contains("node_modules\\")
                || path_str.contains(".git\\")
            {
                continue;
            }

            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let lang = Self::ext_to_language(ext);
                if let Some(lang) = lang {
                    let lines = Self::count_lines(path);
                    let entry = counts.entry(lang.to_string()).or_insert((0, 0));
                    entry.0 += 1;
                    entry.1 += lines;
                }
            }
        }

        let total_lines: u64 = counts.values().map(|(_, l)| l).sum();

        let mut profiles: Vec<LanguageProfile> = counts
            .into_iter()
            .map(|(language, (file_count, line_count))| {
                let percentage = if total_lines > 0 {
                    (line_count as f32 / total_lines as f32) * 100.0
                } else {
                    0.0
                };
                LanguageProfile {
                    language,
                    file_count,
                    line_count,
                    percentage,
                }
            })
            .collect();

        profiles.sort_by_key(|b| std::cmp::Reverse(b.line_count));
        profiles
    }

    fn ext_to_language(ext: &str) -> Option<&'static str> {
        match ext.to_lowercase().as_str() {
            "rs" => Some("Rust"),
            "ts" | "tsx" => Some("TypeScript"),
            "js" | "jsx" | "mjs" | "cjs" => Some("JavaScript"),
            "py" | "pyw" => Some("Python"),
            "go" => Some("Go"),
            "java" => Some("Java"),
            "cs" => Some("C#"),
            "rb" => Some("Ruby"),
            "php" => Some("PHP"),
            "sql" => Some("SQL"),
            "toml" => Some("TOML"),
            "yaml" | "yml" => Some("YAML"),
            "json" => Some("JSON"),
            "md" | "markdown" => Some("Markdown"),
            "html" | "htm" => Some("HTML"),
            "css" | "scss" | "sass" => Some("CSS"),
            _ => None,
        }
    }

    fn count_lines(path: &Path) -> u64 {
        std::fs::read_to_string(path)
            .map(|c| c.lines().count() as u64)
            .unwrap_or(0)
    }
}

// ─────────────────────────────────────────────────────────────────
// Dependency Analyzer
// ─────────────────────────────────────────────────────────────────

pub struct DependencyAnalyzer;

impl DependencyAnalyzer {
    /// Scan for dependency manifests and extract dependency info.
    pub fn analyze(root_path: &Path) -> Vec<DependencyInfo> {
        let mut deps = Vec::new();

        for entry in WalkDir::new(root_path)
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();

            // Skip target/node_modules
            let path_str = path.to_string_lossy();
            if path_str.contains("target/")
                || path_str.contains("node_modules/")
                || path_str.contains("target\\")
                || path_str.contains("node_modules\\")
            {
                continue;
            }

            match name.as_ref() {
                "Cargo.toml" => Self::parse_cargo(path, &mut deps),
                "package.json" => Self::parse_package_json(path, &mut deps),
                "requirements.txt" => Self::parse_requirements(path, &mut deps),
                "go.mod" => Self::parse_go_mod(path, &mut deps),
                _ => {}
            }
        }

        deps
    }

    fn parse_cargo(path: &Path, deps: &mut Vec<DependencyInfo>) {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let table: toml::Table = match content.parse() {
            Ok(v) => v,
            Err(_) => return,
        };
        let source = path.to_string_lossy().to_string();

        // Parse [dependencies]
        if let Some(dep_table) = table.get("dependencies").and_then(|d| d.as_table()) {
            for (name, val) in dep_table {
                let version = Self::extract_cargo_version(val);
                deps.push(DependencyInfo {
                    name: name.clone(),
                    version,
                    dep_type: DependencyType::Runtime,
                    source_file: source.clone(),
                });
            }
        }

        // Parse [dev-dependencies]
        if let Some(dep_table) = table.get("dev-dependencies").and_then(|d| d.as_table()) {
            for (name, val) in dep_table {
                let version = Self::extract_cargo_version(val);
                deps.push(DependencyInfo {
                    name: name.clone(),
                    version,
                    dep_type: DependencyType::Dev,
                    source_file: source.clone(),
                });
            }
        }

        // Parse [build-dependencies]
        if let Some(dep_table) = table.get("build-dependencies").and_then(|d| d.as_table()) {
            for (name, val) in dep_table {
                let version = Self::extract_cargo_version(val);
                deps.push(DependencyInfo {
                    name: name.clone(),
                    version,
                    dep_type: DependencyType::Build,
                    source_file: source.clone(),
                });
            }
        }
    }

    fn extract_cargo_version(val: &toml::Value) -> String {
        match val {
            toml::Value::String(s) => s.clone(),
            toml::Value::Table(t) => t
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("*")
                .to_string(),
            _ => "*".to_string(),
        }
    }

    fn parse_package_json(path: &Path, deps: &mut Vec<DependencyInfo>) {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => return,
        };
        let source = path.to_string_lossy().to_string();

        if let Some(dep_obj) = json.get("dependencies").and_then(|d| d.as_object()) {
            for (name, val) in dep_obj {
                deps.push(DependencyInfo {
                    name: name.clone(),
                    version: val.as_str().unwrap_or("*").to_string(),
                    dep_type: DependencyType::Runtime,
                    source_file: source.clone(),
                });
            }
        }

        if let Some(dep_obj) = json.get("devDependencies").and_then(|d| d.as_object()) {
            for (name, val) in dep_obj {
                deps.push(DependencyInfo {
                    name: name.clone(),
                    version: val.as_str().unwrap_or("*").to_string(),
                    dep_type: DependencyType::Dev,
                    source_file: source.clone(),
                });
            }
        }

        if let Some(dep_obj) = json.get("peerDependencies").and_then(|d| d.as_object()) {
            for (name, val) in dep_obj {
                deps.push(DependencyInfo {
                    name: name.clone(),
                    version: val.as_str().unwrap_or("*").to_string(),
                    dep_type: DependencyType::Peer,
                    source_file: source.clone(),
                });
            }
        }
    }

    fn parse_requirements(path: &Path, deps: &mut Vec<DependencyInfo>) {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let source = path.to_string_lossy().to_string();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // Handle name==version, name>=version, name
            let (name, version) = if let Some(pos) = line.find("==") {
                (&line[..pos], line[pos + 2..].to_string())
            } else if let Some(pos) = line.find(">=") {
                (&line[..pos], format!(">={}", &line[pos + 2..]))
            } else if let Some(pos) = line.find("~=") {
                (&line[..pos], format!("~={}", &line[pos + 2..]))
            } else {
                (line, "*".to_string())
            };

            deps.push(DependencyInfo {
                name: name.trim().to_string(),
                version: version.trim().to_string(),
                dep_type: DependencyType::Runtime,
                source_file: source.clone(),
            });
        }
    }

    fn parse_go_mod(path: &Path, deps: &mut Vec<DependencyInfo>) {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let source = path.to_string_lossy().to_string();
        let mut in_require = false;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("require (") || line == "require (" {
                in_require = true;
                continue;
            }
            if line == ")" {
                in_require = false;
                continue;
            }
            if in_require && !line.is_empty() && !line.starts_with("//") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    deps.push(DependencyInfo {
                        name: parts[0].to_string(),
                        version: parts[1].to_string(),
                        dep_type: DependencyType::Runtime,
                        source_file: source.clone(),
                    });
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────
// Folder Structure Analyzer
// ─────────────────────────────────────────────────────────────────

pub struct FolderAnalyzer;

impl FolderAnalyzer {
    /// Build a tree of the project structure (max depth 3 for portability).
    pub fn analyze(root_path: &Path, max_depth: usize) -> FolderTree {
        let name = root_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let mut root = FolderTree::new_dir(name);
        Self::build_tree(root_path, &mut root, 0, max_depth);
        root
    }

    fn build_tree(dir: &Path, node: &mut FolderTree, depth: usize, max_depth: usize) {
        if depth >= max_depth {
            return;
        }

        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        let mut items: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        items.sort_by_key(|e| e.file_name());

        for entry in items {
            let path = entry.path();
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // Skip hidden dirs, target, node_modules
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }

            if path.is_dir() {
                let mut child = FolderTree::new_dir(&name);
                Self::build_tree(&path, &mut child, depth + 1, max_depth);
                // Count files in this subtree
                child.file_count = child
                    .children
                    .iter()
                    .map(|c| if c.is_dir { c.file_count } else { 1 })
                    .sum();
                node.children.push(child);
            } else {
                node.children.push(FolderTree::new_leaf(&name));
                node.file_count += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn language_analyzer_counts_files() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}\nfn foo() {}\n").unwrap();
        fs::write(dir.path().join("lib.rs"), "pub fn bar() {}\n").unwrap();
        fs::write(dir.path().join("app.ts"), "export const x = 1;\n").unwrap();

        let profiles = LanguageAnalyzer::analyze(dir.path());
        assert!(!profiles.is_empty());
        let rust = profiles.iter().find(|p| p.language == "Rust");
        assert!(rust.is_some());
        assert_eq!(rust.unwrap().file_count, 2);
    }

    #[test]
    fn dependency_analyzer_parses_cargo() {
        let dir = TempDir::new().unwrap();
        let cargo = r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = { version = "1.40", features = ["full"] }

[dev-dependencies]
tempfile = "3.12"
"#;
        fs::write(dir.path().join("Cargo.toml"), cargo).unwrap();

        let parsed: Result<toml::Table, _> = cargo.parse();
        assert!(
            parsed.is_ok(),
            "Failed to parse inline cargo string: {:?}",
            parsed.err()
        );

        let deps = DependencyAnalyzer::analyze(dir.path());
        assert_eq!(deps.len(), 3, "Expected 3 dependencies, found: {:#?}", deps);
        assert!(deps
            .iter()
            .any(|d| d.name == "serde" && d.dep_type == DependencyType::Runtime));
        assert!(deps
            .iter()
            .any(|d| d.name == "tempfile" && d.dep_type == DependencyType::Dev));
    }

    #[test]
    fn dependency_analyzer_parses_package_json() {
        let dir = TempDir::new().unwrap();
        let pkg = r#"{
  "dependencies": { "react": "^18.0.0" },
  "devDependencies": { "vite": "^5.0.0" }
}"#;
        fs::write(dir.path().join("package.json"), pkg).unwrap();

        let deps = DependencyAnalyzer::analyze(dir.path());
        assert_eq!(deps.len(), 2);
        assert!(deps
            .iter()
            .any(|d| d.name == "react" && d.dep_type == DependencyType::Runtime));
    }

    #[test]
    fn architecture_analyzer_detects_modular() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("crates/ares-core")).unwrap();
        fs::create_dir_all(dir.path().join("crates/ares-store")).unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]",
        )
        .unwrap();

        let profile = ArchitectureAnalyzer::analyze(dir.path());
        assert_eq!(profile.style, ArchitectureStyle::Modular);
        assert!(profile.components.len() >= 2);
    }

    #[test]
    fn folder_analyzer_builds_tree() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("README.md"), "# Test").unwrap();

        let tree = FolderAnalyzer::analyze(dir.path(), 3);
        assert!(tree.is_dir);
        assert!(!tree.children.is_empty());
    }

    #[test]
    fn dependency_analyzer_parses_requirements_txt() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("requirements.txt"),
            "flask==2.0.1\nrequests>=2.28.0\nnumpy\n",
        )
        .unwrap();

        let deps = DependencyAnalyzer::analyze(dir.path());
        assert_eq!(deps.len(), 3);
        assert!(deps
            .iter()
            .any(|d| d.name == "flask" && d.version == "2.0.1"));
    }
}
