use crate::forecast::models::{OutcomePrediction, StrategyRanking};
use crate::prediction::models::CounterfactualResult;
use crate::risk::models::RiskReport;
use crate::scenario::models::Scenario;
use crate::simulation::models::SimulationResult;
use serde::{Deserialize, Serialize};

/// Structured, human-readable explanation of a prediction decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionExplanation {
    pub selected_scenario: String,
    pub scenario_type: String,
    pub reason: String,
    pub success_probability: f64,
    pub risk_level: String,
    pub estimated_cost: f64,
    pub estimated_duration_secs: f64,
    pub historical_similar_count: u32,
    pub confidence: f64,
    pub confidence_reasons: Vec<String>,
    pub contributing_factors: Vec<String>,
    pub counterfactual_insights: Vec<String>,
    pub alternative_scenarios: Vec<AlternativeScenario>,
}

/// Brief summary of an alternative scenario considered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeScenario {
    pub scenario_type: String,
    pub composite_score: f64,
    pub reason_not_selected: String,
}

/// Generates human-readable explanations for prediction decisions.
pub struct PredictionExplainer;

impl Default for PredictionExplainer {
    fn default() -> Self {
        Self::new()
    }
}

impl PredictionExplainer {
    pub fn new() -> Self {
        Self
    }

    /// Generate a full explanation for a World Model decision.
    pub fn explain(
        &self,
        scenario: &Scenario,
        simulation: &SimulationResult,
        risk_report: &RiskReport,
        prediction: &OutcomePrediction,
        rankings: &[StrategyRanking],
        counterfactuals: &[CounterfactualResult],
    ) -> PredictionExplanation {
        let contributing_factors =
            self.extract_contributing_factors(scenario, simulation, risk_report, prediction);

        let counterfactual_insights = self.extract_counterfactual_insights(counterfactuals);

        let alternative_scenarios = self.extract_alternatives(rankings, &scenario.id);

        let reason = self.build_reason(scenario, simulation, risk_report, prediction);

        PredictionExplanation {
            selected_scenario: scenario.description.clone(),
            scenario_type: scenario.scenario_type.to_string(),
            reason,
            success_probability: prediction.success_probability,
            risk_level: risk_report.overall_risk.as_str().to_string(),
            estimated_cost: prediction.estimated_cost,
            estimated_duration_secs: prediction.estimated_duration_secs,
            historical_similar_count: prediction.similar_mission_count,
            confidence: prediction.confidence,
            confidence_reasons: prediction.confidence_reasons.clone(),
            contributing_factors,
            counterfactual_insights,
            alternative_scenarios,
        }
    }

    /// Generate a concise, human-readable summary.
    pub fn summarize(&self, explanation: &PredictionExplanation) -> String {
        format!(
            "Selected '{}' strategy: {:.0}% success, {} risk, ${:.2} cost, {:.0}s duration \
             (confidence: {:.0}%, based on {} similar missions). {}",
            explanation.scenario_type,
            explanation.success_probability * 100.0,
            explanation.risk_level,
            explanation.estimated_cost,
            explanation.estimated_duration_secs,
            explanation.confidence * 100.0,
            explanation.historical_similar_count,
            explanation.reason,
        )
    }

    fn build_reason(
        &self,
        scenario: &Scenario,
        simulation: &SimulationResult,
        risk_report: &RiskReport,
        prediction: &OutcomePrediction,
    ) -> String {
        let mut parts = Vec::new();

        if prediction.similar_mission_count > 10 {
            parts.push(format!(
                "Strong historical basis ({} similar missions)",
                prediction.similar_mission_count
            ));
        } else if prediction.similar_mission_count > 0 {
            parts.push(format!(
                "Some historical data ({} similar missions)",
                prediction.similar_mission_count
            ));
        } else {
            parts.push("Novel goal — prediction based on simulation only".to_string());
        }

        if risk_report.is_acceptable() {
            parts.push("Acceptable risk level".to_string());
        } else {
            parts.push(format!(
                "Elevated risk: {}",
                risk_report.overall_risk.as_str()
            ));
        }

        if simulation.success_probability > 0.8 {
            parts.push("High simulation confidence".to_string());
        }

        if scenario.estimated_quality > 0.8 {
            parts.push("High quality target achievable".to_string());
        }

        parts.join(". ")
    }

    fn extract_contributing_factors(
        &self,
        scenario: &Scenario,
        simulation: &SimulationResult,
        risk_report: &RiskReport,
        prediction: &OutcomePrediction,
    ) -> Vec<String> {
        let mut factors = Vec::new();

        factors.push(format!("Step count: {}", scenario.steps.len()));
        factors.push(format!(
            "Agent utilization: {:.0}%",
            simulation.agent_utilization * 100.0
        ));
        factors.push(format!(
            "Risk factors identified: {}",
            risk_report.risk_factors.len()
        ));
        factors.push(format!(
            "Prediction method: {}",
            prediction.prediction_method.as_str()
        ));

        if !risk_report.mitigations.is_empty() {
            factors.push(format!(
                "Mitigations available: {}",
                risk_report.mitigations.len()
            ));
        }

        factors
    }

    fn extract_counterfactual_insights(
        &self,
        counterfactuals: &[CounterfactualResult],
    ) -> Vec<String> {
        counterfactuals
            .iter()
            .filter(|cf| cf.is_significant())
            .map(|cf| {
                format!(
                    "{}: success drops {:.0}% → {:.0}% (impact: {:.2})",
                    cf.counterfactual.description,
                    cf.original_success_probability * 100.0,
                    cf.adjusted_success_probability * 100.0,
                    cf.impact_score,
                )
            })
            .collect()
    }

    fn extract_alternatives(
        &self,
        rankings: &[StrategyRanking],
        selected_id: &ares_core::ScenarioId,
    ) -> Vec<AlternativeScenario> {
        rankings
            .iter()
            .filter(|r| r.scenario_id != *selected_id)
            .take(3)
            .map(|r| {
                let reason = if r.risk_score > 0.5 {
                    "Higher risk".to_string()
                } else if r.cost_score < 0.3 {
                    "Higher cost".to_string()
                } else if r.speed_score < 0.3 {
                    "Slower execution".to_string()
                } else {
                    "Lower composite score".to_string()
                };

                AlternativeScenario {
                    scenario_type: format!("Rank #{}", r.rank),
                    composite_score: r.composite_score,
                    reason_not_selected: reason,
                }
            })
            .collect()
    }
}
