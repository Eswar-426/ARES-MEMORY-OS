use crate::certification::compute_certification;
use crate::compliance_engine::ComplianceEngine;
use crate::diagnostics::{why_is_this_non_compliant, DiagnosticsReport};
use crate::models::{
    ComplianceResult, GovernanceCertification, GovernanceScorecard, PolicyDefinition, PolicyVersion,
};
use crate::policy_loader::PolicyLoader;
use ares_core::{AresError, NodeId, ProjectId};
use ares_store::repositories::graph::SqliteGraphRepository;
use ares_store::db::Store;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

pub struct GovernanceFacade {
    engine: ComplianceEngine<SqliteGraphRepository>,
    policy_loader: PolicyLoader,
    policies: Arc<RwLock<Vec<(PolicyDefinition, PolicyVersion)>>>,
    exemptions_engine: crate::exemptions::ExemptionEngine,
    store: Store,
}

impl GovernanceFacade {
    pub fn new(store: Store, workspace_root: PathBuf) -> Self {
        let graph_repo = SqliteGraphRepository::new(store.clone());
        let engine = ComplianceEngine::new(graph_repo);
        let policy_loader = PolicyLoader::new(workspace_root.clone());

        // Try to load initial policies
        let initial_policies = policy_loader.load_all().unwrap_or_else(|e| {
            debug!("Failed to load initial policies: {}", e);
            Vec::new()
        });

        let exemptions_engine = crate::exemptions::ExemptionEngine::new(workspace_root);

        Self {
            engine,
            policy_loader,
            policies: Arc::new(RwLock::new(initial_policies)),
            exemptions_engine,
            store,
        }
    }

    pub async fn reload_policies(&self) -> Result<usize, AresError> {
        let new_policies = self
            .policy_loader
            .load_all()
            .map_err(|e| AresError::db(format!("Failed to load policies: {}", e)))?;
        let count = new_policies.len();
        let mut w = self.policies.write().await;
        *w = new_policies;
        Ok(count)
    }

    pub async fn get_policies(&self) -> Vec<PolicyVersion> {
        let guard = self.policies.read().await;
        guard.iter().map(|(_, v)| v.clone()).collect()
    }

    pub async fn get_active_policies(&self) -> Vec<(PolicyDefinition, PolicyVersion)> {
        let guard = self.policies.read().await;
        guard.clone()
    }

    pub async fn is_compliant(
        &self,
        project_id: &ProjectId,
        node_id: &NodeId,
    ) -> Result<Vec<ComplianceResult>, AresError> {
        info!("Evaluating compliance for node {}", node_id);
        let r = self.policies.read().await;
        let exemptions = self.exemptions_engine.load_active_exemptions().await.unwrap_or_default();
        self.engine.evaluate_node(project_id, node_id, r.as_slice(), &exemptions)
    }

    pub async fn why_is_this_non_compliant(
        &self,
        project_id: &ProjectId,
        node_id: &NodeId,
    ) -> Result<DiagnosticsReport, AresError> {
        let results = self.is_compliant(project_id, node_id).await?;
        let mut all_violations = Vec::new();
        for r in results {
            all_violations.extend(r.violations);
        }
        Ok(why_is_this_non_compliant(&all_violations))
    }

    pub async fn evaluate_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<ComplianceResult>, AresError> {
        info!("Evaluating compliance for project {}", project_id);
        let r = self.policies.read().await;
        let exemptions = self.exemptions_engine.load_active_exemptions().await.unwrap_or_default();
        self.engine.evaluate_project(project_id, r.as_slice(), &exemptions)
    }

    pub async fn get_scorecard(
        &self,
        project_id: &ProjectId,
    ) -> Result<GovernanceScorecard, AresError> {
        let results = self.evaluate_project(project_id).await?;
        Ok(crate::scorecard::calculate_scorecard(&results))
    }

    pub async fn get_certification(
        &self,
        project_id: &ProjectId,
    ) -> Result<GovernanceCertification, AresError> {
        let results = self.evaluate_project(project_id).await?;
        Ok(compute_certification(project_id.as_str(), &results))
    }

    pub async fn get_dashboard(
        &self,
        project_id: &ProjectId,
    ) -> Result<crate::models::GovernanceDashboard, AresError> {
        let results = self.evaluate_project(project_id).await?;
        let cert = compute_certification(project_id.as_str(), &results);
        let mut top_violations = Vec::new();
        for r in results {
            top_violations.extend(r.violations);
        }

        let req_store = ares_requirements::RequirementStore::new(self.store.clone());
        let mut graph = ares_traceability::TraceabilityGraph::new();
        graph.add_provider(Box::new(ares_requirements::RequirementEdgeProvider::new(self.store.clone())));
        let resolver = ares_requirements::TraceAnalysisEngine::new(&graph);
        let engine = ares_requirements::RequirementCoverageEngine::new();

        let reqs = req_store.list(project_id, ares_requirements::RequirementFilter::default()).unwrap_or_default();
        let coverages: Vec<_> = reqs.iter().map(|req| {
            engine.evaluate(&req.id, &req.status, req.owner.is_some(), &resolver)
        }).collect();

        let (req_summary, top_gaps) = engine.generate_summary(&coverages);

        let mut structural_drift = 0;
        let mut semantic_drift = 0;
        let mut critical_drift = 0;
        let mut unresolved_drift = 0;

        let drift_engine = ares_requirements::RequirementDriftEngine::new(&graph);
        for req in &reqs {
            let baseline = ares_requirements::RequirementBaseline {
                requirement_id: req.id.to_string(),
                approved_at: req.created_at,
                decision_ids: vec![],
                implementation_ids: vec![],
                test_ids: vec![],
                runtime_metrics: vec![],
            };
            if let Some(report) = drift_engine.evaluate_drift(&baseline) {
                unresolved_drift += 1;
                if report.severity == ares_requirements::DriftSeverity::Critical {
                    critical_drift += 1;
                }
                for dt in report.drift_types {
                    match dt {
                        ares_requirements::RequirementDriftType::Structural(_) => structural_drift += 1,
                        ares_requirements::RequirementDriftType::Semantic(_) => semantic_drift += 1,
                    }
                }
            }
        }

        let requirement_drift = crate::models::RequirementDriftSummary {
            structural_drift,
            semantic_drift,
            critical_drift,
            unresolved_drift,
        };

        let requirement_coverage_trend = ares_requirements::RequirementCoverageTrend {
            previous_coverage: req_summary.average_coverage,
            current_coverage: req_summary.average_coverage,
            delta: 0.0,
        };

        let evolution = crate::models::EvolutionMetrics {
            total_requirement_events: 0,
            requirements_changed_this_week: 0,
            requirements_regressed: 0,
            requirements_improved: 0,
        };
        
        let knowledge_gaps = ares_requirements::KnowledgeGapEngine::new(&graph).evaluate_gaps();

        Ok(crate::dashboard::DashboardGenerator::generate_dashboard(
            &cert, 
            top_violations, 
            req_summary, 
            requirement_coverage_trend, 
            requirement_drift,
            evolution,
            top_gaps,
            &knowledge_gaps
        ))
    }

    pub async fn detect_drift(
        &self,
        project_id: &ProjectId,
    ) -> Result<crate::models::PolicyDriftStatus, AresError> {
        let detector = crate::drift::DriftDetector::new(Arc::new(self.store.clone()));
        let policies = self.policies.read().await;
        detector.detect_drift(project_id, &policies).await
    }
    pub async fn get_exemptions(&self) -> Result<Vec<crate::models::PolicyExemption>, AresError> {
        self.exemptions_engine.load_active_exemptions().await
    }
}
