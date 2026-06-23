use crate::models::{ComplianceViolation, PolicyExemption};
use ares_core::AresError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{error, info, warn};

#[derive(Debug, Serialize, Deserialize)]
struct ExemptionWrapper {
    exemption: PolicyExemption,
}

pub struct ExemptionEngine {
    project_root: PathBuf,
}

impl ExemptionEngine {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    pub fn get_exemptions_dir(&self) -> PathBuf {
        self.project_root.join(".governance").join("exemptions")
    }

    pub async fn load_active_exemptions(&self) -> Result<Vec<PolicyExemption>, AresError> {
        let dir = self.get_exemptions_dir();
        if !dir.exists() || !dir.is_dir() {
            return Ok(Vec::new());
        }

        let mut exemptions = Vec::new();
        let mut entries = tokio::fs::read_dir(&dir)
            .await
            .map_err(AresError::Io)?;

        let now = chrono::Utc::now();

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("yaml") {
                match tokio::fs::read_to_string(&path).await {
                    Ok(content) => {
                        let content_str = content.as_str();
                        match serde_yaml::from_str::<ExemptionWrapper>(content_str) {
                            Ok(wrapper) => {
                                let ex = wrapper.exemption;
                                if ex.expires_at > now {
                                    exemptions.push(ex);
                                } else {
                                    info!(
                                        "Exemption {} expired on {}, skipping.",
                                        ex.id, ex.expires_at
                                    );
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse exemption file {:?}: {}", path, e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to read exemption file {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(exemptions)
    }

    pub fn filter_violations(
        &self,
        violations: Vec<ComplianceViolation>,
        exemptions: &[PolicyExemption],
    ) -> Vec<ComplianceViolation> {
        violations
            .into_iter()
            .filter(|v| {
                // Check if violation is exempted
                let is_exempted = exemptions.iter().any(|ex| {
                    let matches_rule =
                        ex.target_rules.is_empty() || ex.target_rules.contains(&v.policy_name);
                    let matches_node =
                        ex.target_nodes.is_empty() || ex.target_nodes.contains(&v.node_id);

                    // If both rule and node constraints are empty, it's a global exemption (unlikely, but handled).
                    // Usually an exemption targets specific rules or specific nodes or both.
                    if ex.target_rules.is_empty() && ex.target_nodes.is_empty() {
                        false // Don't allow blank exemptions to bypass everything
                    } else {
                        matches_rule && matches_node
                    }
                });

                !is_exempted
            })
            .collect()
    }
}
