use super::models::{RankingWeights, StrategyRanking};
use crate::risk::models::RiskReport;
use crate::scenario::models::Scenario;
use crate::simulation::models::SimulationResult;

/// Ranks strategies using a composite scoring formula.
///
/// Score = (quality_weight * quality)
///       + (success_weight * success_probability)
///       - (risk_weight * risk_score)
///       - (cost_weight * normalized_cost)
///       + (speed_weight * (1 - normalized_duration))
///
/// Fully deterministic — no LLM dependency.
pub struct StrategyRanker;

impl Default for StrategyRanker {
    fn default() -> Self {
        Self::new()
    }
}

impl StrategyRanker {
    pub fn new() -> Self {
        Self
    }

    /// Rank a set of scenarios by composite score.
    ///
    /// Returns rankings sorted best-first (highest composite score).
    pub fn rank(
        &self,
        scenarios: &[Scenario],
        simulations: &[SimulationResult],
        risk_reports: &[RiskReport],
        weights: &RankingWeights,
    ) -> Vec<StrategyRanking> {
        // Compute normalization ranges
        let max_cost = simulations
            .iter()
            .map(|s| s.total_cost)
            .fold(f64::NEG_INFINITY, f64::max)
            .max(1.0);
        let max_duration = simulations
            .iter()
            .map(|s| s.task_duration_secs)
            .fold(f64::NEG_INFINITY, f64::max)
            .max(1.0);

        let mut rankings: Vec<StrategyRanking> = scenarios
            .iter()
            .filter_map(|scenario| {
                let sim = simulations.iter().find(|s| s.scenario_id == scenario.id)?;
                let risk = risk_reports.iter().find(|r| r.scenario_id == scenario.id);

                let ranking =
                    self.score_scenario(scenario, sim, risk, weights, max_cost, max_duration);
                Some(ranking)
            })
            .collect();

        // Sort by composite score descending
        rankings.sort_by(|a, b| {
            b.composite_score
                .partial_cmp(&a.composite_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Assign ranks
        for (i, ranking) in rankings.iter_mut().enumerate() {
            ranking.rank = (i + 1) as u32;
        }

        rankings
    }

    /// Score a single scenario.
    fn score_scenario(
        &self,
        scenario: &Scenario,
        simulation: &SimulationResult,
        risk_report: Option<&RiskReport>,
        weights: &RankingWeights,
        max_cost: f64,
        max_duration: f64,
    ) -> StrategyRanking {
        let quality_score = scenario.estimated_quality;
        let success_score = simulation.success_probability;

        let risk_score = risk_report
            .map(|r| r.overall_score())
            .unwrap_or(simulation.risk_score);

        let cost_score = 1.0 - (simulation.total_cost / max_cost).min(1.0);
        let speed_score = 1.0 - (simulation.task_duration_secs / max_duration).min(1.0);

        let composite_score = weights.quality * quality_score + weights.success * success_score
            - weights.risk * risk_score
            + weights.cost * cost_score
            + weights.speed * speed_score;

        let explanation = self.explain_ranking(
            scenario,
            simulation,
            risk_report,
            quality_score,
            success_score,
            risk_score,
            cost_score,
            speed_score,
            composite_score,
        );

        StrategyRanking {
            scenario_id: scenario.id.clone(),
            rank: 0, // assigned after sorting
            composite_score,
            speed_score,
            quality_score,
            cost_score,
            risk_score,
            success_score,
            explanation,
        }
    }

    /// Generate a human-readable explanation for a ranking.
    #[allow(clippy::too_many_arguments)]
    fn explain_ranking(
        &self,
        scenario: &Scenario,
        simulation: &SimulationResult,
        risk_report: Option<&RiskReport>,
        quality_score: f64,
        success_score: f64,
        risk_score: f64,
        cost_score: f64,
        speed_score: f64,
        composite_score: f64,
    ) -> String {
        let risk_level = risk_report
            .map(|r| r.overall_risk.as_str().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        format!(
            "Strategy '{}': composite={:.3}, success={:.0}%, quality={:.0}%, \
             risk={} ({:.2}), cost=${:.2} (score={:.2}), \
             duration={:.0}s (speed={:.2})",
            scenario.scenario_type,
            composite_score,
            success_score * 100.0,
            quality_score * 100.0,
            risk_level,
            risk_score,
            simulation.total_cost,
            cost_score,
            simulation.task_duration_secs,
            speed_score,
        )
    }

    /// Get the best-ranked scenario ID (rank 1).
    pub fn best_scenario_id<'a>(
        &self,
        rankings: &'a [StrategyRanking],
    ) -> Option<&'a ares_core::ScenarioId> {
        rankings.first().map(|r| &r.scenario_id)
    }
}
