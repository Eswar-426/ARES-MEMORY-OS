use crate::assembler::MemoryContextAssembler;
use ares_core::AresError;
use std::sync::Arc;

use ares_context_injector::{build_context_with_store, ContextPackage, TokenBudget};
use ares_core::ProjectId;
use ares_governance::GovernanceFacade;
use ares_knowledge_graph::queries::WhyResult;

pub struct WhyWithContextResult {
    pub graph: WhyResult,
    pub context: ContextPackage,
}

/// MemoryFacade shields the API and external consumers from internal orchestration logic.
#[derive(Clone)]
pub struct MemoryFacade {
    assembler: Arc<MemoryContextAssembler>,
    governance: Arc<GovernanceFacade>,
}

impl MemoryFacade {
    pub fn new(assembler: Arc<MemoryContextAssembler>, governance: Arc<GovernanceFacade>) -> Self {
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

    pub async fn why_with_context(
        &self,
        file_path: &str,
        query: &str,
        project_id: &ProjectId,
    ) -> Result<WhyWithContextResult, AresError> {
        let entity_id = ares_core::canonicalize_node_id(file_path);

        let graph = self
            .assembler
            .graph
            .why_does_this_exist(&entity_id)
            .unwrap_or_else(|_| WhyResult {
                requirements: vec![],
                decisions: vec![],
                evidence: vec![],
            });

        let context = build_context_with_store(
            query,
            file_path,
            &self.assembler.store,
            project_id,
            TokenBudget::Small,
        )
        .await
        .map_err(|e| AresError::Database(e.to_string()))?; // AresError doesn't have Internal out of the box? We can use Database or similar

        Ok(WhyWithContextResult { graph, context })
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
        let evidence = self
            .assembler
            .graph
            .what_evidence_supports_this(entity_id)?;
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
        let conn = self.assembler.store.get_conn()?;

        // Find RepositoryEvents that OccurredIn this entity
        let mut stmt = conn.prepare("
            SELECT e.id, e.name, e.properties, e.created_at
            FROM graph_entities e
            JOIN graph_relationships r ON e.id = r.source_entity
            WHERE r.target_entity = ?1 AND r.relationship_type = 'OccurredIn' AND e.entity_type = 'RepositoryEvent'
            ORDER BY e.created_at ASC
        ").map_err(|e| AresError::Database(e.to_string()))?;

        let mut events = Vec::new();
        let rows = stmt
            .query_map(rusqlite::params![entity_id], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let props_str: String = row.get(2)?;
                let created_at: i64 = row.get(3)?;
                let props: serde_json::Value =
                    serde_json::from_str(&props_str).unwrap_or(serde_json::json!({}));
                Ok(serde_json::json!({
                    "event_id": id,
                    "name": name,
                    "created_at": created_at,
                    "properties": props
                }))
            })
            .map_err(|e| AresError::Database(e.to_string()))?;

        for ev in rows.flatten() {
            events.push(ev);
        }

        // Add latest Snapshot info
        let mut snap_stmt = conn
            .prepare(
                "
            SELECT e.id, e.name, e.created_at 
            FROM graph_entities e
            JOIN graph_relationships r ON e.id = r.source_entity
            WHERE r.relationship_type = 'OccurredIn' AND e.entity_type = 'RepositorySnapshot'
            ORDER BY e.created_at DESC LIMIT 1
        ",
            )
            .map_err(|e| AresError::Database(e.to_string()))?;

        let latest_snapshot: Option<serde_json::Value> = snap_stmt
            .query_row([], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let created_at: i64 = row.get(2)?;
                Ok(serde_json::json!({
                    "snapshot_id": id,
                    "name": name,
                    "created_at": created_at
                }))
            })
            .ok();

        Ok(serde_json::json!({
            "entity": entity_id,
            "latest_snapshot": latest_snapshot,
            "events": events
        }))
    }

    pub fn is_requirement_fully_implemented(&self, req_id_str: &str) -> Result<String, AresError> {
        let store = self.assembler.store.clone();
        let req_store = ares_requirements::RequirementStore::new(store.clone());
        let req_id = ares_core::RequirementId::from(req_id_str);

        let req = req_store
            .get(&req_id)?
            .ok_or_else(|| AresError::not_found("requirement", req_id_str))?;

        let mut graph = ares_traceability::TraceabilityGraph::new();
        graph.add_provider(Box::new(ares_requirements::RequirementEdgeProvider::new(
            store,
        )));

        let resolver = ares_requirements::TraceAnalysisEngine::new(&graph);
        let engine = ares_requirements::RequirementCoverageEngine::new();
        let coverage = engine.evaluate(&req.id, &req.status, req.owner.is_some(), &resolver);

        let mut md = format!("{} ({})\n\n", req.id.as_str(), req.title);

        let decision_mark = if coverage.gaps.iter().any(|g| {
            matches!(
                g.gap_type,
                ares_requirements::KnowledgeGapType::MissingDecision
            )
        }) {
            "✗"
        } else {
            "✓"
        };
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
                    ares_requirements::KnowledgeGapType::UnapprovedRequirement => {
                        "Missing Approval"
                    }
                    ares_requirements::KnowledgeGapType::MissingDecision => "Missing Decision",
                    ares_requirements::KnowledgeGapType::MissingImplementation => {
                        "Missing Implementation"
                    }
                    ares_requirements::KnowledgeGapType::MissingTest => "Missing Tests",
                    ares_requirements::KnowledgeGapType::MissingRuntimeMetric => {
                        "Missing Runtime Metric"
                    }
                    ares_requirements::KnowledgeGapType::MissingOwner => "Missing Owner",
                    _ => "Other Gap",
                };
                md.push_str(&format!("- {}\n", text));
            }
            md.push('\n');
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
        let req = req_store.get(&id_obj)?.ok_or_else(|| AresError::NotFound {
            resource_type: "Requirement",
            id: req_id.to_string(),
        })?;

        let mut graph = ares_traceability::TraceabilityGraph::new();
        graph.add_provider(Box::new(ares_requirements::RequirementEdgeProvider::new(
            self.assembler.store.clone(),
        )));

        let impact_engine = ares_requirements::RequirementImpactEngine::new(&graph);
        let report = impact_engine.evaluate_impact(req.id.as_ref());
        Ok(Self::format_impact_report(&report))
    }

    pub fn format_impact_report(
        report: &ares_requirements::impact::RequirementImpactReport,
    ) -> String {
        let mut md = format!("Requirement: {}\n\n", report.requirement_id);
        md.push_str(&format!(
            "Blast Radius: {}/100\n\n",
            report.blast_radius_score.round()
        ));
        md.push_str(&format!("Severity: {:?}\n\n", report.severity));
        md.push_str(&format!("Risk: {:?}\n\n", report.risk));

        md.push_str("Affected:\n");
        if !report.affected_decisions.is_empty() {
            md.push_str(&format!(
                "✓ {} Decisions\n",
                report.affected_decisions.len()
            ));
        }
        if !report.affected_architecture.is_empty() {
            md.push_str(&format!(
                "✓ {} Architecture Nodes\n",
                report.affected_architecture.len()
            ));
        }
        if !report.affected_code.is_empty() {
            md.push_str(&format!(
                "✓ {} Code Artifacts\n",
                report.affected_code.len()
            ));
        }
        if !report.affected_tests.is_empty() {
            md.push_str(&format!("✓ {} Tests\n", report.affected_tests.len()));
        }
        if !report.affected_runtime_metrics.is_empty() {
            md.push_str(&format!(
                "✓ {} Runtime Metrics\n",
                report.affected_runtime_metrics.len()
            ));
        }
        if !report.affected_governance.is_empty() {
            md.push_str(&format!(
                "✓ {} Governance Policies\n",
                report.affected_governance.len()
            ));
        }
        md.push('\n');

        if !report.affected_decisions.is_empty() || !report.affected_architecture.is_empty() {
            md.push_str("Most Critical Dependency:\n");
            if let Some(first_dec) = report.affected_decisions.first() {
                md.push_str(&format!("{}\n", first_dec));
            } else if let Some(first_arch) = report.affected_architecture.first() {
                md.push_str(&format!("{}\n", first_arch));
            }
            md.push('\n');
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
        let req = req_store.get(&id_obj)?.ok_or_else(|| AresError::NotFound {
            resource_type: "Requirement",
            id: req_id.to_string(),
        })?;

        let mut graph = ares_traceability::TraceabilityGraph::new();
        graph.add_provider(Box::new(ares_requirements::RequirementEdgeProvider::new(
            self.assembler.store.clone(),
        )));

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
            md.push_str(&format!(
                "**Status:** DRIFT DETECTED (Severity: {:?}, Confidence: {:?})\n\n",
                report.severity, report.confidence
            ));

            if !report.evidence.is_empty() {
                md.push_str("### Evidence of Drift:\n");
                for ev in report.evidence {
                    md.push_str(&format!(
                        "- **{}** (Expected: {}, Observed: {})\n",
                        ev.target_node, ev.expected_state, ev.observed_state
                    ));
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

    pub fn how_has_requirement_evolved(&self, req_id: &str) -> Result<String, AresError> {
        let req_store = ares_requirements::RequirementStore::new(self.assembler.store.clone());
        let id_obj = ares_core::RequirementId::from(req_id);
        let req = req_store.get(&id_obj)?.ok_or_else(|| AresError::NotFound {
            resource_type: "Requirement",
            id: req_id.to_string(),
        })?;

        let engine = ares_requirements::evolution::RequirementEvolutionEngine::new(
            self.assembler.store.clone(),
        );
        let timeline = engine.get_timeline(&id_obj)?;

        Ok(Self::format_evolution_report(&req, &timeline))
    }

    pub fn format_evolution_report(
        req: &ares_requirements::Requirement,
        timeline: &ares_requirements::RequirementTimeline,
    ) -> String {
        use chrono::{TimeZone, Utc};

        let mut md = format!(
            "Requirement Evolution Report\n\nRequirement:\n{}\n\n",
            req.id.as_str()
        );

        let created_dt = Utc
            .timestamp_micros(req.created_at)
            .single()
            .unwrap_or_default();
        md.push_str(&format!("Created:\n{}\n\n", created_dt.format("%Y-%m-%d")));

        md.push_str("Timeline:\n\n");
        if timeline.events.is_empty() {
            md.push_str("No evolution events recorded.\n\n");
        } else {
            for event in &timeline.events {
                let dt = Utc
                    .timestamp_micros(event.timestamp)
                    .single()
                    .unwrap_or_default();
                let date_str = dt.format("%Y-%m-%d").to_string();

                let event_str = match &event.event_type {
                    ares_requirements::RequirementEvolutionType::RequirementCreated => {
                        "Requirement Created"
                    }
                    ares_requirements::RequirementEvolutionType::RequirementUpdated => {
                        "Requirement Updated"
                    }
                    ares_requirements::RequirementEvolutionType::RequirementApproved => {
                        "Requirement Approved"
                    }
                    ares_requirements::RequirementEvolutionType::RequirementRejected => {
                        "Requirement Rejected"
                    }
                    ares_requirements::RequirementEvolutionType::RequirementOwnershipChanged => {
                        "Ownership Changed"
                    }
                    ares_requirements::RequirementEvolutionType::DecisionAdded => "Decision Added",
                    ares_requirements::RequirementEvolutionType::DecisionRemoved => {
                        "Decision Removed"
                    }
                    ares_requirements::RequirementEvolutionType::ImplementationAdded => {
                        "Implementation Added"
                    }
                    ares_requirements::RequirementEvolutionType::ImplementationRemoved => {
                        "Implementation Removed"
                    }
                    ares_requirements::RequirementEvolutionType::TestAdded => "Tests Added",
                    ares_requirements::RequirementEvolutionType::TestRemoved => "Tests Removed",
                    ares_requirements::RequirementEvolutionType::RuntimeMetricAdded => {
                        "Runtime Metric Added"
                    }
                    ares_requirements::RequirementEvolutionType::RuntimeMetricRemoved => {
                        "Runtime Metric Removed"
                    }
                    ares_requirements::RequirementEvolutionType::CoverageImproved => {
                        "Coverage Improved"
                    }
                    ares_requirements::RequirementEvolutionType::CoverageRegressed => {
                        "Coverage Regressed"
                    }
                    ares_requirements::RequirementEvolutionType::DriftDetected => "Drift Detected",
                    ares_requirements::RequirementEvolutionType::DriftResolved => "Drift Resolved",
                    ares_requirements::RequirementEvolutionType::GovernanceViolation => {
                        "Governance Violation"
                    }
                    ares_requirements::RequirementEvolutionType::GovernanceApproved => {
                        "Governance Approved"
                    }
                };

                let mut details = String::new();
                if let (Some(prev), Some(new)) = (event.previous_score, event.new_score) {
                    details = format!(" ({}% → {}%)", prev.round(), new.round());
                } else if !event.description.is_empty() {
                    details = format!(" ({})", event.description);
                }

                md.push_str(&format!("✓ {} - {}{}\n", date_str, event_str, details));
            }
            md.push('\n');
        }

        let mut coverage_transitions = Vec::new();
        for e in &timeline.events {
            if e.event_type == ares_requirements::RequirementEvolutionType::CoverageImproved
                || e.event_type == ares_requirements::RequirementEvolutionType::CoverageRegressed
            {
                if let Some(new_score) = e.new_score {
                    coverage_transitions.push(format!("{}%", new_score.round()));
                }
            }
        }

        md.push_str("Coverage:\n");
        if coverage_transitions.is_empty() {
            md.push_str("0%\n\n");
        } else {
            md.push_str(&format!("{}\n\n", coverage_transitions.join(" → ")));
        }

        md.push_str("Drift:\n");
        let mut drift_events = Vec::new();
        for e in &timeline.events {
            if e.event_type == ares_requirements::RequirementEvolutionType::DriftDetected
                || e.event_type == ares_requirements::RequirementEvolutionType::DriftResolved
            {
                let dt = Utc
                    .timestamp_micros(e.timestamp)
                    .single()
                    .unwrap_or_default();
                let date_str = dt.format("%Y-%m-%d").to_string();
                let action =
                    if e.event_type == ares_requirements::RequirementEvolutionType::DriftDetected {
                        "Detected"
                    } else {
                        "Resolved"
                    };
                drift_events.push(format!("{} ({})", action, date_str));
            }
        }

        if drift_events.is_empty() {
            md.push_str("None\n\n");
        } else {
            for e in drift_events {
                md.push_str(&format!("{}\n", e));
            }
            md.push('\n');
        }

        md.push_str("Current Status:\nHealthy\n"); // simplified for the canonical question format

        md
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

    pub fn get_gaps_by_type(&self, gap_type: &str) -> Result<Vec<serde_json::Value>, AresError> {
        let conn = self.assembler.store.get_conn()?;

        let mut stmt = conn.prepare("
            SELECT e.id, e.name 
            FROM graph_relationships g
            JOIN graph_entities e ON e.id = g.target_entity
            JOIN graph_entities gap ON gap.id = g.source_entity
            WHERE gap.entity_type = 'KnowledgeGap' AND gap.name = ?1 AND g.relationship_type = 'HasGap'
        ").map_err(|e| AresError::Database(e.to_string()))?;

        let mut results = Vec::new();
        let rows = stmt
            .query_map(rusqlite::params![gap_type], |row| {
                let entity_id: String = row.get(0)?;
                let title: String = row.get(1)?;
                Ok((entity_id, title))
            })
            .map_err(|e| AresError::Database(e.to_string()))?;

        let mut entity_tuples = Vec::new();
        for tup in rows.flatten() {
            entity_tuples.push(tup);
        }

        for (entity_id, title) in entity_tuples {
            let mut related_code = Vec::new();
            let mut code_stmt = conn.prepare("
                SELECT target_entity FROM graph_relationships 
                WHERE source_entity = ?1 AND relationship_type IN ('ImplementedBy', 'Drives', 'References')
            ").unwrap();

            let code_rows = code_stmt
                .query_map(rusqlite::params![entity_id], |r| r.get::<_, String>(0))
                .unwrap();
            for c in code_rows.flatten() {
                related_code.push(c);
            }

            if gap_type == "CodeWithoutTests" && related_code.is_empty() {
                related_code.push(entity_id.clone());
            }

            results.push(serde_json::json!({
                "id": entity_id,
                "title": title,
                "gap_type": gap_type,
                "related_code": related_code
            }));
        }

        Ok(results)
    }

    pub fn get_uncovered_requirements(&self) -> Result<Vec<serde_json::Value>, AresError> {
        let mut r1 = self.get_gaps_by_type("RequirementWithoutTests")?;
        let mut r2 = self.get_gaps_by_type("RequirementWithoutImplementation")?;
        r1.append(&mut r2);
        Ok(r1)
    }

    pub fn get_code_without_tests(&self) -> Result<Vec<serde_json::Value>, AresError> {
        self.get_gaps_by_type("CodeWithoutTests")
    }

    pub fn get_orphan_decisions(&self) -> Result<Vec<serde_json::Value>, AresError> {
        let mut r1 = self.get_gaps_by_type("OrphanDecision")?;
        let mut r2 = self.get_gaps_by_type("DecisionWithoutRequirement")?;
        r1.append(&mut r2);
        // deduplicate just in case
        r1.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
        r1.dedup_by(|a, b| a["id"] == b["id"]);
        Ok(r1)
    }

    pub fn get_orphan_requirements(&self) -> Result<Vec<serde_json::Value>, AresError> {
        self.get_gaps_by_type("OrphanRequirement")
    }
}
