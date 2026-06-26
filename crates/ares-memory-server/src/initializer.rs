use crate::{config::AresConfig, manifest::BuildManifest};
use ares_core::AresError;
use std::fs;
use std::path::Path;

pub struct RepositoryInitializer;

impl RepositoryInitializer {
    pub fn init(path: &Path) -> Result<(), AresError> {
        let ares_dir = path.join(".ares");
        if !ares_dir.exists() {
            fs::create_dir_all(&ares_dir).map_err(|e| AresError::validation(e.to_string()))?;
        }

        let config_path = ares_dir.join("config.toml");
        if !config_path.exists() {
            let _default_config = AresConfig::default();
            // We just use a hardcoded string here if toml isn't available to keep dependencies simple
            let config_str = r#"
[repository]
id = "local-repo"
name = "My Repository"
version = "1.0.0"

[storage]
path = ".ares/memory.db"

[scanners]
exclude = ["node_modules", "target", ".git"]
            "#
            .trim();
            fs::write(config_path, config_str).map_err(|e| AresError::validation(e.to_string()))?;
        }

        let manifest_path = ares_dir.join("build_manifest.json");
        if !manifest_path.exists() {
            let manifest = BuildManifest {
                version: "1.16.0".to_string(),
                ..Default::default()
            };
            let manifest_str = serde_json::to_string_pretty(&manifest).unwrap();
            fs::write(manifest_path, manifest_str)
                .map_err(|e| AresError::validation(e.to_string()))?;
        }

        Ok(())
    }
}
