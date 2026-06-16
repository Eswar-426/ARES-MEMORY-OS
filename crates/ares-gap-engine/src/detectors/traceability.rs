use super::GapDetector;
use crate::models::{DetectionMethod, Gap, GapSeverity, GapType};
use ares_core::{AresError, id::{ProjectId, new_id}};
use ares_store::Store;
use ares_traceability::{EdgeProvider, TraceTargetType};
use ares_decision_intelligence::storage::DecisionEdgeProvider;
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

pub struct TraceabilityGapDetector;

#[async_trait]
impl GapDetector for TraceabilityGapDetector {
    fn supported_types(&self) -> Vec<GapType> {
        vec![
            GapType::OrphanCode,
        ]
    }

    async fn detect(&self, project_id: &ProjectId, store: Arc<Store>) -> Result<Vec<Gap>, AresError> {
        let mut gaps = Vec::new();
        
        let decision_edges = DecisionEdgeProvider::new((*store).clone());
        let edges = decision_edges.edges()?;

        // Count targets
        let mut code_targets = 0;
        for edge in edges {
            if edge.target_type == TraceTargetType::Code {
                code_targets += 1;
            }
        }

        // Just a placeholder rule for now: if we have zero traces to code in the whole repo, 
        // we might flag a global gap. In reality, we would scan `ares-code` nodes and 
        // flag any nodes that lack incoming edges from Requirements or Decisions.
        
        if code_targets == 0 {
            gaps.push(Gap {
                id: format!("gap_trace_code_{}", new_id()),
                project_id: project_id.clone(),
                gap_type: GapType::OrphanCode,
                description: "The project has 0 traces mapping decisions to code implementation.".to_string(),
                source_id: project_id.as_str().to_string(),
                detection_method: DetectionMethod::Statistical,
                evidence_score: 1.0,
                severity: GapSeverity::Warning,
                identified_at: Utc::now().timestamp_micros(),
                metadata: HashMap::new(),
                evidence: vec![],
                reason: None,
                priority_score: None,
                impact_radius: None,
            });
        }

        Ok(gaps)
    }
}
