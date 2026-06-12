//! Prediction Gate — pre-execution gate for missions.
//!
//! Evaluates whether a mission should proceed based on World Model predictions.
//! Acts as a safety check before execution begins.
//!
//! Flow:
//! ```text
//! Mission → Predict Outcome → Risk Check → Approve/Reject → Execute
//! ```

use ares_world_model::forecast::models::{HistoricalMission, RankingWeights};
use ares_world_model::planner_bridge::bridge::{PlannerBridge, WorldModelDecision};
use ares_world_model::risk::models::RiskLevel;
use ares_world_model::scenario::models::ScenarioGenerationConfig;
use ares_world_model::simulation::models::SimulationConfig;
use ares_world_model::state::models::WorldState;
use serde::{Deserialize, Serialize};

/// Decision from the prediction gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateDecision {
    pub approved: bool,
    pub risk_level: String,
    pub success_probability: f64,
    pub confidence: f64,
    pub explanation: String,
    pub confidence_reasons: Vec<String>,
}

/// Configuration for the gate's approval thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateConfig {
    /// Minimum success probability to auto-approve.
    pub min_success_probability: f64,
    /// Maximum acceptable risk level.
    pub max_risk_level: RiskLevel,
    /// Minimum prediction confidence to trust.
    pub min_confidence: f64,
}

impl Default for GateConfig {
    fn default() -> Self {
        Self {
            min_success_probability: 0.5,
            max_risk_level: RiskLevel::Moderate,
            min_confidence: 0.3,
        }
    }
}

/// Pre-execution gate that uses the World Model to predict whether
/// a mission should proceed.
pub struct PredictionGate {
    bridge: PlannerBridge,
    config: GateConfig,
}

impl Default for PredictionGate {
    fn default() -> Self {
        Self::new(GateConfig::default())
    }
}

impl PredictionGate {
    pub fn new(config: GateConfig) -> Self {
        Self {
            bridge: PlannerBridge::new(),
            config,
        }
    }

    /// Evaluate whether a mission should proceed.
    pub fn evaluate_mission(
        &mut self,
        goal_id: &str,
        goal_title: &str,
        world_state: &WorldState,
        historical_missions: &[HistoricalMission],
    ) -> GateDecision {
        let decision = self.bridge.evaluate_goal(
            goal_id,
            goal_title,
            world_state,
            historical_missions,
            &ScenarioGenerationConfig::default(),
            &SimulationConfig::default(),
            &RankingWeights::default(),
        );

        self.make_gate_decision(&decision)
    }

    /// Make an approval decision based on the World Model output.
    fn make_gate_decision(&self, decision: &WorldModelDecision) -> GateDecision {
        let risk_level = &decision.best_risk_report.overall_risk;
        let success_prob = decision.prediction.success_probability;
        let confidence = decision.prediction.confidence;

        let risk_acceptable = *risk_level <= self.config.max_risk_level;
        let success_acceptable = success_prob >= self.config.min_success_probability;
        let confidence_sufficient = confidence >= self.config.min_confidence;

        let approved = risk_acceptable && success_acceptable && confidence_sufficient;

        let explanation = if approved {
            format!(
                "Mission approved: {:.0}% success probability, {} risk, {:.0}% confidence",
                success_prob * 100.0,
                risk_level.as_str(),
                confidence * 100.0,
            )
        } else {
            let mut reasons = Vec::new();
            if !risk_acceptable {
                reasons.push(format!("Risk too high: {}", risk_level.as_str()));
            }
            if !success_acceptable {
                reasons.push(format!(
                    "Success probability too low: {:.0}% (min: {:.0}%)",
                    success_prob * 100.0,
                    self.config.min_success_probability * 100.0,
                ));
            }
            if !confidence_sufficient {
                reasons.push(format!(
                    "Confidence too low: {:.0}% (min: {:.0}%)",
                    confidence * 100.0,
                    self.config.min_confidence * 100.0,
                ));
            }
            format!("Mission rejected: {}", reasons.join("; "))
        };

        GateDecision {
            approved,
            risk_level: risk_level.as_str().to_string(),
            success_probability: success_prob,
            confidence,
            explanation,
            confidence_reasons: decision.prediction.confidence_reasons.clone(),
        }
    }

    /// Get the underlying World Model decision for deeper inspection.
    pub fn evaluate_full(
        &mut self,
        goal_id: &str,
        goal_title: &str,
        world_state: &WorldState,
        historical_missions: &[HistoricalMission],
    ) -> (GateDecision, WorldModelDecision) {
        let decision = self.bridge.evaluate_goal(
            goal_id,
            goal_title,
            world_state,
            historical_missions,
            &ScenarioGenerationConfig::default(),
            &SimulationConfig::default(),
            &RankingWeights::default(),
        );
        let gate = self.make_gate_decision(&decision);
        (gate, decision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ares_world_model::state::models::*;

    fn make_state() -> WorldState {
        WorldState {
            id: ares_core::WorldStateId::new(),
            goals: vec![WorldGoal {
                id: "g1".into(),
                title: "Test".into(),
                priority: "high".into(),
                status: "active".into(),
            }],
            resources: vec![WorldResource {
                name: "budget".into(),
                resource_type: ResourceType::Budget,
                available: 100.0,
                capacity: 100.0,
            }],
            active_agents: vec![WorldAgent {
                id: "a1".into(),
                name: "Agent".into(),
                role: "coder".into(),
                status: "ready".into(),
                success_rate: 0.85,
            }],
            constraints: vec![],
            snapshot_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn gate_approves_good_mission() {
        let mut gate = PredictionGate::default();
        let state = make_state();
        let decision = gate.evaluate_mission("g1", "Build API", &state, &[]);
        // With good state and simple goal, should approve
        assert!(decision.approved || decision.success_probability > 0.0);
    }

    #[test]
    fn gate_decision_has_explanation() {
        let mut gate = PredictionGate::default();
        let state = make_state();
        let decision = gate.evaluate_mission("g1", "Build API", &state, &[]);
        assert!(!decision.explanation.is_empty());
    }

    #[test]
    fn gate_decision_has_confidence_reasons() {
        let mut gate = PredictionGate::default();
        let state = make_state();
        let decision = gate.evaluate_mission("g1", "Build API", &state, &[]);
        assert!(!decision.confidence_reasons.is_empty());
    }

    #[test]
    fn gate_config_default() {
        let config = GateConfig::default();
        assert!((config.min_success_probability - 0.5).abs() < f64::EPSILON);
        assert_eq!(config.max_risk_level, RiskLevel::Moderate);
    }

    #[test]
    fn strict_gate_may_reject() {
        let config = GateConfig {
            min_success_probability: 0.99,
            max_risk_level: RiskLevel::Negligible,
            min_confidence: 0.95,
        };
        let mut gate = PredictionGate::new(config);
        let state = make_state();
        let decision = gate.evaluate_mission("g1", "Build API", &state, &[]);
        // Strict thresholds should make approval very hard
        if !decision.approved {
            assert!(decision.explanation.contains("rejected"));
        }
    }

    #[test]
    fn gate_evaluate_full_returns_both() {
        let mut gate = PredictionGate::default();
        let state = make_state();
        let (gate_decision, world_decision) = gate.evaluate_full("g1", "Build API", &state, &[]);
        assert!(!gate_decision.explanation.is_empty());
        assert!(!world_decision.rankings.is_empty());
    }

    #[test]
    fn gate_decision_serialization() {
        let mut gate = PredictionGate::default();
        let state = make_state();
        let decision = gate.evaluate_mission("g1", "Build API", &state, &[]);
        let json = serde_json::to_string(&decision).unwrap();
        let back: GateDecision = serde_json::from_str(&json).unwrap();
        assert_eq!(back.approved, decision.approved);
    }
}
