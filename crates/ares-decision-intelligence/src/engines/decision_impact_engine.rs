use ares_evolution::impact_engine::MemoryImpactEngine;
use ares_core::types::impact::ChangeImpactReport;
use ares_core::types::staleness::StalenessFinding;

pub struct DecisionImpactEngine {
    impact_engine: MemoryImpactEngine,
}

impl DecisionImpactEngine {
    pub fn new(impact_engine: MemoryImpactEngine) -> Self {
        Self { impact_engine }
    }

    pub async fn calculate_decision_impact(
        &self,
        project_id: &str,
        decision_id: &str,
        impacted_graph_nodes: &[(String, String)],
        impacted_files: Vec<String>,
        impacted_owners: Vec<String>,
        staleness_findings: &[StalenessFinding],
    ) -> Result<ChangeImpactReport, String> {
        self.impact_engine.analyze_impact(
            project_id,
            decision_id,
            impacted_graph_nodes,
            impacted_files,
            impacted_owners,
            staleness_findings,
        ).await
    }
}
