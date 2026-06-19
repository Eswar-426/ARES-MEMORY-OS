use crate::models::{PolicyDefinition, PolicyDriftStatus, PolicyVersion};
use ares_core::{AresError, ProjectId};
use ares_store::Store;
use std::collections::HashMap;
use std::sync::Arc;

pub struct DriftDetector {
    store: Arc<Store>,
}

impl DriftDetector {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    pub async fn detect_drift(
        &self,
        project_id: &ProjectId,
        active_policies: &[(PolicyDefinition, PolicyVersion)],
    ) -> Result<PolicyDriftStatus, AresError> {
        let conn = self.store.get_conn()?;

        // Get the latest evaluation checksums for each policy for this project
        // We'll join compliance_results to policy_versions to get the policy name and checksum
        let mut stmt = conn.prepare(
            r#"
            SELECT pv.policy_name, pv.checksum 
            FROM compliance_results cr
            JOIN policy_versions pv ON cr.policy_version_id = pv.checksum
            WHERE cr.project_id = ?
            GROUP BY pv.policy_name
            "#,
        ).map_err(|e| AresError::Database(e.to_string()))?;

        let mut historical_checksums = HashMap::new();
        let mut rows = stmt
            .query([project_id.as_str()])
            .map_err(|e| AresError::Database(e.to_string()))?;

        while let Some(row) = rows.next().map_err(|e| AresError::Database(e.to_string()))? {
            let policy_name: String = row.get(0).unwrap_or_default();
            let checksum: String = row.get(1).unwrap_or_default();
            historical_checksums.insert(policy_name, checksum);
        }

        let mut outdated_policies = Vec::new();

        for (def, version) in active_policies {
            if let Some(historical_checksum) = historical_checksums.get(&def.metadata.name) {
                if historical_checksum != &version.checksum {
                    outdated_policies.push(def.metadata.name.clone());
                }
            } else {
                // If the policy is totally new and hasn't been evaluated against the project yet
                outdated_policies.push(def.metadata.name.clone());
            }
        }

        let drift_detected = !outdated_policies.is_empty();

        Ok(PolicyDriftStatus {
            project_id: project_id.to_string(),
            drift_detected,
            outdated_policies,
        })
    }
}
