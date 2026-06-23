use crate::models::{PolicyDefinition, PolicyVersion};
use anyhow::{Context, Result};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

pub struct PolicyLoader {
    policy_dir: PathBuf,
}

impl PolicyLoader {
    pub fn new<P: AsRef<Path>>(workspace_root: P) -> Self {
        let policy_dir = workspace_root.as_ref().join(".governance").join("policies");
        Self { policy_dir }
    }

    pub fn load_all(&self) -> Result<Vec<(PolicyDefinition, PolicyVersion)>> {
        if !self.policy_dir.exists() {
            warn!(
                "Governance policy directory not found at {:?}",
                self.policy_dir
            );
            return Ok(Vec::new());
        }

        let mut policies = Vec::new();

        for entry in fs::read_dir(&self.policy_dir).context("Failed to read policy directory")? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("yaml") {
                debug!("Loading policy from {:?}", path);
                match self.load_policy(&path) {
                    Ok((def, version)) => policies.push((def, version)),
                    Err(e) => warn!("Failed to load policy {:?}: {:#?}", path, e),
                }
            }
        }

        info!("Loaded {} governance policies", policies.len());
        Ok(policies)
    }

    fn load_policy(&self, path: &Path) -> Result<(PolicyDefinition, PolicyVersion)> {
        let content = fs::read_to_string(path).context("Failed to read policy file")?;

        let def: PolicyDefinition =
            serde_yaml::from_str(&content).context("Failed to parse policy YAML")?;

        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let hash = hasher.finalize();
        let checksum = format!("{:x}", hash);

        let version = PolicyVersion {
            policy_name: def.metadata.name.clone(),
            version: def.metadata.version.clone(),
            checksum,
            loaded_at: Utc::now().timestamp(),
        };

        Ok((def, version))
    }
}
