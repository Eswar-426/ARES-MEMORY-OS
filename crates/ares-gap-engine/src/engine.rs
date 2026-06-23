use crate::detectors::GapDetector;
use crate::models::RepositoryHealthReport;
use ares_core::{id::ProjectId, AresError};
use ares_decision_intelligence::health::DecisionHealthEngine;
use ares_requirements::health::RequirementHealthEngine;
use ares_store::Store;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

pub struct GapEngine {
    store: Arc<Store>,
    detectors: Vec<Box<dyn GapDetector>>,
}

impl GapEngine {
    pub fn new(store: Arc<Store>) -> Self {
        Self {
            store,
            detectors: Vec::new(),
        }
    }

    pub fn register_detector(&mut self, detector: Box<dyn GapDetector>) {
        self.detectors.push(detector);
    }

    pub async fn run_scan(
        &self,
        project_id: &ProjectId,
    ) -> Result<RepositoryHealthReport, AresError> {
        info!("Starting gap detection scan for project {}", project_id);
        let mut all_gaps = Vec::new();

        // 1. Run all registered gap detectors
        for detector in &self.detectors {
            let gaps = detector.detect(project_id, self.store.clone()).await?;
            all_gaps.extend(gaps);
        }

        // 2. Fetch component-level health scores from the domain crates
        let mut component_scores = HashMap::new();

        let req_engine = RequirementHealthEngine::new((*self.store).clone());
        if let Ok(req_health) = req_engine.compute_health(project_id) {
            component_scores.insert("requirements".to_string(), req_health.total_score);
        }

        let dec_engine = DecisionHealthEngine::new((*self.store).clone());
        if let Ok(dec_health) = dec_engine.generate_snapshot(project_id) {
            component_scores.insert("decisions".to_string(), dec_health.health_score as f64);
        }

        // 3. Calculate an overall unified score.
        // If domain scores exist, average them. Then apply penalties based on gaps.
        let mut base_score = 100.0;
        if !component_scores.is_empty() {
            let sum: f64 = component_scores.values().sum();
            base_score = sum / (component_scores.len() as f64);
        }

        // Apply a small penalty for each gap found outside the domains' own scores,
        // or just rely on domain scores. For this prototype, we'll subtract 1.0 point
        // per gap, down to a floor of 0.0.
        let penalty = all_gaps.len() as f64 * 1.5;
        let overall_score = (base_score - penalty).max(0.0);

        // --- GAP INTELLIGENCE PIPELINE ---

        // 1. Reason about gaps (Attach Root Cause and Evidence)
        let reasoner = crate::reasoning::GapReasoner::new(self.store.clone());
        let reasoned_gaps = reasoner.reason(all_gaps)?;

        // 2. Prioritize gaps (Compute Impact Radius and Priority Score)
        let prioritizer = crate::prioritization::GapPrioritizer::new(self.store.clone());
        let prioritized_gaps = prioritizer.prioritize(reasoned_gaps)?;

        // 3. Cluster gaps (Group by Root Cause)
        let cluster_engine = crate::clustering::GapClusterEngine::new();
        let clusters = cluster_engine.cluster(&prioritized_gaps);

        // 4. Calculate Knowledge Debt
        let stale_entities = prioritized_gaps
            .iter()
            .filter(|g| g.gap_type == crate::models::GapType::StaleRequirement)
            .count();
        let orphan_entities = prioritized_gaps
            .iter()
            .filter(|g| g.gap_type == crate::models::GapType::OrphanCode)
            .count();
        let critical_gaps = prioritized_gaps
            .iter()
            .filter(|g| g.severity == crate::models::GapSeverity::Critical)
            .count();

        let knowledge_debt = crate::models::KnowledgeDebt {
            debt_score: (critical_gaps as f64 * 10.0) + (prioritized_gaps.len() as f64 * 2.0),
            total_gaps: prioritized_gaps.len(),
            critical_gaps,
            stale_entities,
            orphan_entities,
        };

        // 5. Generate Snapshot and Save Trends
        let health_snapshot = crate::models::RepositoryHealthSnapshot {
            snapshot_id: ares_core::id::new_id(),
            project_id: project_id.clone(),
            snapshot_time: Utc::now().timestamp_micros(),
            overall_score,
            component_scores,
            total_gaps: prioritized_gaps.len(),
            critical_gaps,
        };

        let trend_engine = crate::trends::HealthTrendEngine::new(self.store.clone());
        if let Err(e) = trend_engine.save_snapshot(&health_snapshot) {
            tracing::warn!("Failed to save health trend snapshot: {:?}", e);
        }

        // --- END INTELLIGENCE PIPELINE ---

        let report = RepositoryHealthReport {
            project_id: project_id.clone(),
            generated_at: Utc::now().timestamp_micros(),
            health: health_snapshot,
            gaps: prioritized_gaps.clone(),
            clusters,
            prioritized_gaps,
            knowledge_debt,
        };

        info!(
            "Gap scan completed for {}. Found {} gaps, {} clusters. Overall score: {:.2}",
            project_id,
            report.gaps.len(),
            report.clusters.len(),
            report.health.overall_score
        );

        Ok(report)
    }
}
