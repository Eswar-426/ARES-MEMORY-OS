use ares_core::types::impact::{ChangeImpactReport, ImpactSeverity};
use ares_core::types::staleness::StalenessFinding;
use ares_store::repositories::drift::DriftRepository;
use std::sync::Arc;

pub struct MemoryImpactEngine {
    drift_repo: Arc<dyn DriftRepository>,
}

impl MemoryImpactEngine {
    pub fn new(drift_repo: Arc<dyn DriftRepository>) -> Self {
        Self { drift_repo }
    }

    pub async fn analyze_impact(
        &self,
        project_id: &str,
        target_node_id: &str,
        impacted_graph_nodes: &[(String, String)], // (NodeID, NodeType)
        impacted_files: Vec<String>,
        impacted_owners: Vec<String>,
        staleness_findings: &[StalenessFinding],
    ) -> Result<ChangeImpactReport, String> {
        let drifts = self
            .drift_repo
            .get_candidates_for_project(project_id)
            .await
            .map_err(|e| e.to_string())?;

        let mut reqs = Vec::new();
        let mut decs = Vec::new();
        let mut arch = Vec::new();

        for (n_id, n_type) in impacted_graph_nodes {
            match n_type.to_lowercase().as_str() {
                "requirement" => reqs.push(n_id.clone()),
                "decision" => decs.push(n_id.clone()),
                "architecture" => arch.push(n_id.clone()),
                _ => {}
            }
        }

        let mut drift_risk = 0.0;
        let mut staleness_risk = 0.0;
        let mut rationale = Vec::new();

        // Calculate drift risk from active drift candidates hitting the impacted nodes
        let mut matching_drifts = 0;
        for drift in &drifts {
            if impacted_graph_nodes
                .iter()
                .any(|(id, _)| id == &drift.target_node_id)
            {
                matching_drifts += 1;
                rationale.push(format!(
                    "Drift Candidate {} affects impacted node {}",
                    drift.id, drift.target_node_id
                ));
            }
        }

        if matching_drifts > 0 {
            drift_risk = (matching_drifts as f32 * 25.0).min(100.0);
        }

        let mut matching_staleness = 0;
        let mut staleness_penalty = 0.0;
        for finding in staleness_findings {
            if impacted_graph_nodes
                .iter()
                .any(|(id, _)| id == &finding.node_id)
            {
                matching_staleness += 1;
                // Higher staleness (lower score) = higher risk
                let risk = 100.0 - finding.score;
                staleness_penalty += risk;
                rationale.push(format!(
                    "Staleness Finding on node {} adds risk {:.1}",
                    finding.node_id, risk
                ));
            }
        }

        if matching_staleness > 0 {
            staleness_risk = (staleness_penalty / matching_staleness as f32).min(100.0);
        }

        // Base structural impact based on how many things are hit
        let structural_impact = ((reqs.len() * 30) + (decs.len() * 20) + (arch.len() * 15)) as f32;
        let structural_impact = structural_impact.min(100.0);

        let total_impact_score =
            (structural_impact * 0.4) + (drift_risk * 0.3) + (staleness_risk * 0.3);

        let severity = match total_impact_score {
            s if s >= 80.0 => ImpactSeverity::Critical,
            s if s >= 50.0 => ImpactSeverity::High,
            s if s >= 20.0 => ImpactSeverity::Medium,
            _ => ImpactSeverity::Low,
        };

        rationale.push(format!(
            "Total Impact Score {:.1} classifies as {:?}",
            total_impact_score, severity
        ));

        Ok(ChangeImpactReport {
            target_node_id: target_node_id.to_string(),
            project_id: project_id.to_string(),
            impacted_requirements: reqs,
            impacted_decisions: decs,
            impacted_architecture: arch,
            impacted_files,
            impacted_owners,
            drift_risk,
            staleness_risk,
            total_impact_score,
            severity,
            rationale,
        })
    }
}
