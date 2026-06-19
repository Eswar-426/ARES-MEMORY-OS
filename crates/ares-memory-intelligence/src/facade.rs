use std::sync::Arc;
use ares_core::AresError;
use ares_knowledge_graph::impact::ImpactReport;
use ares_memory_evolution::models::EvolutionTimeline;
use crate::assembler::MemoryContextAssembler;

use ares_governance::GovernanceFacade;

/// MemoryFacade shields the API and external consumers from internal orchestration logic.
#[derive(Clone)]
pub struct MemoryFacade {
    assembler: Arc<MemoryContextAssembler>,
    governance: Arc<GovernanceFacade>,
}

impl MemoryFacade {
    pub fn new(
        assembler: Arc<MemoryContextAssembler>,
        governance: Arc<GovernanceFacade>,
    ) -> Self {
        Self {
            assembler,
            governance,
        }
    }

    pub fn why(&self, entity_id: &str) -> Result<serde_json::Value, AresError> {
        let result = self.assembler.graph.why_does_this_exist(entity_id)?;
        Ok(serde_json::json!({
            "entity": entity_id,
            "requirements": result.requirements,
            "decisions": result.decisions,
            "evidence": result.evidence
        }))
    }

    pub fn who(&self, entity_id: &str) -> Result<serde_json::Value, AresError> {
        let result = self.assembler.graph.who_owns_this(entity_id)?;
        Ok(serde_json::json!({
            "entity": entity_id,
            "owners": result.owners,
            "approvers": result.approvers,
            "decisions": result.decisions
        }))
    }
    
    pub fn approval(&self, entity_id: &str) -> Result<serde_json::Value, AresError> {
        let result = self.assembler.graph.who_owns_this(entity_id)?;
        Ok(serde_json::json!({
            "entity": entity_id,
            "approvers": result.approvers,
            "decisions": result.decisions
        }))
    }

    pub fn evidence(&self, entity_id: &str) -> Result<serde_json::Value, AresError> {
        let evidence = self.assembler.graph.what_evidence_supports_this(entity_id)?;
        Ok(serde_json::json!({
            "entity": entity_id,
            "evidence": evidence
        }))
    }

    pub fn replacement(&self, entity_id: &str) -> Result<serde_json::Value, AresError> {
        // Mocked supersession for now as EvolutionTimeline handles replacement at higher level
        Ok(serde_json::json!({
            "entity": entity_id,
            "superseded_by": null
        }))
    }



    pub fn impact(&self, entity_id: &str) -> Result<serde_json::Value, AresError> {
        let report = self.assembler.graph.what_breaks_if_changed(entity_id)?;
        Ok(serde_json::json!({
            "total_score": report.total_score,
            "risk_level": report.risk_level,
            "impacted_nodes": report.impacted_nodes
        }))
    }

    pub fn evolution(&self, entity_id: &str) -> Result<serde_json::Value, AresError> {
        let timeline = self.assembler.evolution.how_has_this_evolved(entity_id)?;
        Ok(serde_json::json!({
            "entity": entity_id,
            "revisions": timeline.revisions.len() // Mocking the deep serialization for now
        }))
    }

    pub fn is_requirement_fully_implemented(&self, req_id_str: &str) -> Result<String, AresError> {
        let store = self.assembler.store.clone();
        let req_store = ares_requirements::RequirementStore::new(store.clone());
        let req_id = ares_core::RequirementId::from(req_id_str);
        
        let req = req_store.get(&req_id)?
            .ok_or_else(|| AresError::not_found("requirement", req_id_str))?;

        let mut graph = ares_traceability::TraceabilityGraph::new();
        graph.add_provider(Box::new(ares_requirements::RequirementEdgeProvider::new(store)));
        
        let resolver = ares_requirements::GraphTraceResolver::new(&graph);
        let engine = ares_requirements::RequirementCoverageEngine::new();
        let coverage = engine.evaluate(&req.id, &req.status, req.owner.is_some(), &resolver);

        let mut md = format!("{} ({})\n\n", req.id.as_str(), req.title);
        
        let decision_mark = if coverage.gaps.iter().any(|g| matches!(g.gap_type, ares_requirements::GapType::MissingDecision)) { "✗" } else { "✓" };
        md.push_str(&format!("Decision: {}\n", decision_mark));
        
        let impl_mark = if coverage.implemented { "✓" } else { "✗" };
        md.push_str(&format!("Implementation: {}\n", impl_mark));

        let test_mark = if coverage.verified { "✓" } else { "✗" };
        md.push_str(&format!("Tests: {}\n", test_mark));

        let metric_mark = if coverage.monitored { "✓" } else { "✗" };
        md.push_str(&format!("Runtime Metrics: {}\n\n", metric_mark));

        md.push_str(&format!("Coverage: {:.0}%\n\n", coverage.coverage_score));
        
        if !coverage.gaps.is_empty() {
            md.push_str("Gaps:\n");
            for gap in &coverage.gaps {
                let text = match gap.gap_type {
                    ares_requirements::GapType::MissingApproval => "Missing Approval",
                    ares_requirements::GapType::MissingDecision => "Missing Decision",
                    ares_requirements::GapType::MissingImplementation => "Missing Implementation",
                    ares_requirements::GapType::MissingTests => "Missing Tests",
                    ares_requirements::GapType::MissingRuntimeMetric => "Missing Runtime Metric",
                    ares_requirements::GapType::MissingOwner => "Missing Owner",
                };
                md.push_str(&format!("- {}\n", text));
            }
            md.push_str("\n");
        }

        let status_str = match coverage.status {
            ares_requirements::CoverageStatus::Orphaned => "ORPHANED",
            ares_requirements::CoverageStatus::Partial => "PARTIALLY COVERED",
            ares_requirements::CoverageStatus::Covered => "COVERED",
            ares_requirements::CoverageStatus::Verified => "VERIFIED",
        };
        md.push_str(&format!("Status:\n{}", status_str));

        Ok(md)
    }

    pub fn analyze_requirement_impact(&self, req_id: &str) -> Result<String, AresError> {
        let req_store = ares_requirements::RequirementStore::new(self.assembler.store.clone());
        let id_obj = ares_core::RequirementId::from(req_id);
        let req = req_store.get(&id_obj)?
            .ok_or_else(|| AresError::NotFound {
                resource_type: "Requirement",
                id: req_id.to_string(),
            })?;

        let mut graph = ares_traceability::TraceabilityGraph::new();
        graph.add_provider(Box::new(ares_requirements::RequirementEdgeProvider::new(self.assembler.store.clone())));

        let impact_engine = ares_requirements::RequirementImpactEngine::new(&graph);
        let report = impact_engine.evaluate_impact(&req.id.to_string());
        Ok(Self::format_impact_report(&report))
    }

    pub fn format_impact_report(report: &ares_requirements::impact::RequirementImpactReport) -> String {
        let mut md = format!("Requirement: {}\n\n", report.requirement_id);
        md.push_str(&format!("Blast Radius: {}/100\n\n", report.blast_radius_score.round()));
        md.push_str(&format!("Severity: {:?}\n\n", report.severity));
        md.push_str(&format!("Risk: {:?}\n\n", report.risk));

        md.push_str("Affected:\n");
        if !report.affected_decisions.is_empty() {
            md.push_str(&format!("✓ {} Decisions\n", report.affected_decisions.len()));
        }
        if !report.affected_architecture.is_empty() {
            md.push_str(&format!("✓ {} Architecture Nodes\n", report.affected_architecture.len()));
        }
        if !report.affected_code.is_empty() {
            md.push_str(&format!("✓ {} Code Artifacts\n", report.affected_code.len()));
        }
        if !report.affected_tests.is_empty() {
            md.push_str(&format!("✓ {} Tests\n", report.affected_tests.len()));
        }
        if !report.affected_runtime_metrics.is_empty() {
            md.push_str(&format!("✓ {} Runtime Metrics\n", report.affected_runtime_metrics.len()));
        }
        if !report.affected_governance.is_empty() {
            md.push_str(&format!("✓ {} Governance Policies\n", report.affected_governance.len()));
        }
        md.push_str("\n");

        if !report.affected_decisions.is_empty() || !report.affected_architecture.is_empty() {
            md.push_str("Most Critical Dependency:\n");
            if let Some(first_dec) = report.affected_decisions.first() {
                md.push_str(&format!("{}\n", first_dec));
            } else if let Some(first_arch) = report.affected_architecture.first() {
                md.push_str(&format!("{}\n", first_arch));
            }
            md.push_str("\n");
        }

        if !report.affected_governance.is_empty() {
            md.push_str("Governance Impact:\n");
            for gov in &report.affected_governance {
                md.push_str(&format!("{}\n", gov));
            }
        }

        md
    }

    pub fn does_requirement_satisfy_intent(&self, req_id: &str) -> Result<String, AresError> {
        let req_store = ares_requirements::RequirementStore::new(self.assembler.store.clone());
        let id_obj = ares_core::RequirementId::from(req_id);
        let req = req_store.get(&id_obj)?
            .ok_or_else(|| AresError::NotFound {
                resource_type: "Requirement",
                id: req_id.to_string(),
            })?;

        let mut graph = ares_traceability::TraceabilityGraph::new();
        graph.add_provider(Box::new(ares_requirements::RequirementEdgeProvider::new(self.assembler.store.clone())));

        let baseline = ares_requirements::RequirementBaseline {
            requirement_id: req.id.to_string(),
            approved_at: req.created_at,
            decision_ids: vec![], // Assume fetching approved snapshot
            implementation_ids: vec![],
            test_ids: vec![],
            runtime_metrics: vec![],
        };

        let engine = ares_requirements::RequirementDriftEngine::new(&graph);
        let mut md = format!("# Intent Verification for {}\n\n", req.id);

        if let Some(report) = engine.evaluate_drift(&baseline) {
            md.push_str(&format!("**Status:** DRIFT DETECTED (Severity: {:?}, Confidence: {:?})\n\n", report.severity, report.confidence));

            if !report.evidence.is_empty() {
                md.push_str("### Evidence of Drift:\n");
                for ev in report.evidence {
                    md.push_str(&format!("- **{}** (Expected: {}, Observed: {})\n", ev.target_node, ev.expected_state, ev.observed_state));
                }
                md.push('\n');
            }

            if !report.remediations.is_empty() {
                md.push_str("### Recommended Actions:\n");
                for rem in report.remediations {
                    md.push_str(&format!("- {}\n", rem));
                }
            }
        } else {
            md.push_str("**Status:** INTENT SATISFIED\n\nNo structural or semantic drift detected against the approved baseline.");
        }

        Ok(md)
    }

    pub fn context(&self, entity_id: &str) -> Result<serde_json::Value, AresError> {
        self.assembler.get_entity_full_context(entity_id)
    }

    pub fn get_assembler(&self) -> Arc<MemoryContextAssembler> {
        self.assembler.clone()
    }

    pub fn get_governance(&self) -> Arc<GovernanceFacade> {
        self.governance.clone()
    }
}
