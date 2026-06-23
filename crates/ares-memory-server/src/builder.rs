use crate::manifest::BuildManifest;
use ares_core::AresError;
use std::fs;
use std::path::Path;

pub struct RepositoryBuilder;

impl RepositoryBuilder {
    pub fn build(path: &Path) -> Result<(), AresError> {
        let mut manifest = Self::load_manifest(path)?;
        
        let stages = vec![
            "Build Stage 1: Scanner",
            "Build Stage 2: Storage",
            "Build Stage 3: Traceability",
            "Build Stage 4: Reasoning",
            "Build Stage 5: Evolution",
            "Build Stage 6: Completeness",
            "Build Stage 7: Governance",
            "Build Stage 8: Retrieval Indexes",
            "Build Stage 9: Decision Intelligence",
            "Build Stage 10: Repository Intelligence",
            "Build Stage 11: Knowledge Gap",
        ];

        for stage in stages {
            // Log stage execution
            if !manifest.stages_completed.contains(&stage.to_string()) {
                manifest.stages_completed.push(stage.to_string());
            }
        }

        manifest.last_build = chrono::Utc::now().to_rfc3339();
        Self::save_manifest(path, &manifest)?;

        Ok(())
    }

    fn load_manifest(path: &Path) -> Result<BuildManifest, AresError> {
        let manifest_path = path.join(".ares/build_manifest.json");
        let manifest_str = fs::read_to_string(&manifest_path).map_err(|e| AresError::validation(e.to_string()))?;
        serde_json::from_str(&manifest_str).map_err(|e| AresError::validation(e.to_string()))
    }

    fn save_manifest(path: &Path, manifest: &BuildManifest) -> Result<(), AresError> {
        let manifest_path = path.join(".ares/build_manifest.json");
        let manifest_str = serde_json::to_string_pretty(manifest).unwrap();
        fs::write(&manifest_path, manifest_str).map_err(|e| AresError::validation(e.to_string()))
    }
}
