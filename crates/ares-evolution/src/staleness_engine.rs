use ares_core::id::NodeId;
use ares_core::types::evolution::{EvolutionEvent, EvolutionEventType};
use ares_core::types::staleness::{HealthClassification, StalenessFactors, StalenessFinding};
use ares_store::repositories::evolution::EvolutionRepository;
use chrono::Utc;
use std::sync::Arc;

pub struct StalenessEngine {
    repo: Arc<dyn EvolutionRepository>,
}

impl StalenessEngine {
    pub fn new(repo: Arc<dyn EvolutionRepository>) -> Self {
        Self { repo }
    }

    pub async fn analyze(
        &self,
        project_id: &str,
        node_id: &str,
        node_type: &str,
        factors: &StalenessFactors,
        previous_classification: Option<HealthClassification>,
    ) -> Result<Option<StalenessFinding>, String> {
        let w_age = 0.40;
        let w_deps = 0.25;
        let w_vol = 0.20;
        let w_churn = 0.15;

        let multiplier = match node_type.to_lowercase().as_str() {
            "requirement" => 0.5,
            "decision" => 1.0,
            "architecture" => 2.0,
            "code" => 0.0,
            _ => 1.0,
        };

        if multiplier == 0.0 {
            // Code never decays
            let finding = StalenessFinding {
                node_id: node_id.to_string(),
                project_id: project_id.to_string(),
                score: 100.0,
                classification: HealthClassification::Healthy,
                rationale: vec!["Code cannot become stale".to_string()],
            };
            return Ok(Some(finding));
        }

        let decay_age = (factors.age_days as f32 / 10.0) * multiplier;
        let age_score = (100.0 - decay_age).max(0.0);

        let dep_score = (100.0 - (factors.dependent_nodes as f32 * 2.0)).max(0.0);
        let churn_score = (100.0 - (factors.ownership_changes as f32 * 10.0)).max(0.0);
        let vol_score = (100.0 - (factors.downstream_changes as f32 * 5.0)).max(0.0);

        let final_score = (age_score * w_age)
            + (dep_score * w_deps)
            + (vol_score * w_vol)
            + (churn_score * w_churn);

        let classification = match final_score {
            s if s >= 90.0 => HealthClassification::Healthy,
            s if s >= 70.0 => HealthClassification::Aging,
            s if s >= 40.0 => HealthClassification::Stale,
            _ => HealthClassification::Critical,
        };

        let rationale = vec![
            format!("Age: {} days", factors.age_days),
            format!("Dependencies: {}", factors.dependent_nodes),
            format!("Ownership Changes: {}", factors.ownership_changes),
            format!("Volatility: {}", factors.downstream_changes),
            format!("Final Score: {:.0}", final_score),
        ];

        let finding = StalenessFinding {
            node_id: node_id.to_string(),
            project_id: project_id.to_string(),
            score: final_score,
            classification: classification.clone(),
            rationale,
        };

        // If classification drops (or is requested to be checked), emit event
        if let Some(prev) = previous_classification {
            if prev != classification {
                let event = EvolutionEvent {
                    id: NodeId::new(),
                    target_node: NodeId::from(node_id),
                    event_type: EvolutionEventType::StalenessDetected,
                    occurred_at: Utc::now().timestamp_micros(),
                    actor: Some("ARES Staleness Engine".to_string()),
                    rationale: Some(format!(
                        "Health classification changed from {:?} to {:?}",
                        prev, classification
                    )),
                    evidence_ids: vec![],
                    confidence: 1.0,
                };

                self.repo
                    .record_event(project_id, &event)
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }

        Ok(Some(finding))
    }
}
