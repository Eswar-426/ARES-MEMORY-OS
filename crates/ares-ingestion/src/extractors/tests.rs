use std::collections::HashMap;
use std::path::PathBuf;

pub struct TestResolutionEngine;

impl TestResolutionEngine {
    pub fn extract_test_relations(files: &[PathBuf]) -> Vec<(PathBuf, PathBuf)> {
        let mut relations = Vec::new();

        // Build a map of file_stem -> Vec<PathBuf> for code files
        // E.g., "payment" -> ["src/payment.rs", "server/payment.ts"]
        let mut base_names: HashMap<String, Vec<PathBuf>> = HashMap::new();

        for file in files {
            if let Some(stem) = file.file_stem().and_then(|s| s.to_str()) {
                // Ignore obvious test files in the base map to avoid test mapping to test
                if !stem.ends_with("_test")
                    && !stem.ends_with(".spec")
                    && !stem.ends_with(".test")
                    && !stem.starts_with("test_")
                {
                    base_names
                        .entry(stem.to_string())
                        .or_default()
                        .push(file.clone());
                }
            }
        }

        for file in files {
            let _file_str = file.to_string_lossy();
            let file_name = file.file_name().and_then(|s| s.to_str()).unwrap_or("");

            let mut resolved_base = None;
            let mut expected_ext = "";

            // Rust
            if file_name.ends_with("_test.rs") {
                resolved_base = Some(file_name.replace("_test.rs", ""));
                expected_ext = "rs";
            }
            // TypeScript
            else if file_name.ends_with(".spec.ts") {
                resolved_base = Some(file_name.replace(".spec.ts", ""));
                expected_ext = "ts";
            } else if file_name.ends_with(".test.ts") {
                resolved_base = Some(file_name.replace(".test.ts", ""));
                expected_ext = "ts";
            }
            // JavaScript
            else if file_name.ends_with(".spec.js") {
                resolved_base = Some(file_name.replace(".spec.js", ""));
                expected_ext = "js";
            } else if file_name.ends_with(".test.js") {
                resolved_base = Some(file_name.replace(".test.js", ""));
                expected_ext = "js";
            }
            // Python
            else if file_name.ends_with("_test.py") {
                resolved_base = Some(file_name.replace("_test.py", ""));
                expected_ext = "py";
            } else if file_name.starts_with("test_") && file_name.ends_with(".py") {
                resolved_base = Some(file_name.replace("test_", "").replace(".py", ""));
                expected_ext = "py";
            }

            if let Some(base) = resolved_base {
                if let Some(candidates) = base_names.get(&base) {
                    for candidate in candidates {
                        if candidate.extension().and_then(|s| s.to_str()) == Some(expected_ext) {
                            relations.push((candidate.clone(), file.clone())); // code -> test
                        }
                    }
                }
            }
        }

        relations
    }
}
