use crate::evaluation::models::{grade_from_score, EvaluationMetric, MetricScore, MissionScore};
use crate::reflection::mission_reflection::MissionReflection;
use chrono::Utc;

/// Computes mission quality scores across 6 evaluation dimensions.
pub struct SelfEvaluationEngine;

impl Default for SelfEvaluationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfEvaluationEngine {
    pub fn new() -> Self {
        Self
    }

    /// Evaluate a mission across all metrics and produce a composite score.
    pub fn evaluate_mission(&self, reflection: &MissionReflection) -> MissionScore {
        let metrics = vec![
            self.compute_completeness(reflection),
            self.compute_correctness(reflection),
            self.compute_efficiency(reflection),
            self.compute_cost_efficiency(reflection),
            self.compute_safety(reflection),
            self.compute_confidence(reflection),
        ];

        let weighted_sum: f64 = metrics.iter().map(|m| m.score * m.weight).sum();
        let weight_total: f64 = metrics.iter().map(|m| m.weight).sum();
        let overall = if weight_total > 0.0 {
            (weighted_sum / weight_total).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let grade = grade_from_score(overall);

        MissionScore {
            mission_id: reflection.mission_id,
            overall_score: overall,
            metric_scores: metrics,
            evaluated_at: Utc::now(),
            grade,
        }
    }

    /// Completeness = completed_tasks / total_tasks
    pub fn compute_completeness(&self, r: &MissionReflection) -> MetricScore {
        let score = if r.total_tasks == 0 {
            0.0
        } else {
            r.completed_tasks as f64 / r.total_tasks as f64
        };

        MetricScore {
            metric: EvaluationMetric::Completeness,
            score,
            weight: EvaluationMetric::Completeness.default_weight(),
            details: format!("{}/{} tasks completed", r.completed_tasks, r.total_tasks),
        }
    }

    /// Correctness = (completed - failed) / total, weighted by avg agent quality.
    pub fn compute_correctness(&self, r: &MissionReflection) -> MetricScore {
        let base = if r.total_tasks == 0 {
            0.0
        } else {
            (r.completed_tasks as f64 - r.failed_tasks as f64 * 0.5).max(0.0) / r.total_tasks as f64
        };

        // Boost from average agent quality
        let avg_quality = if r.agent_effectiveness.is_empty() {
            1.0
        } else {
            let sum: f64 = r
                .agent_effectiveness
                .values()
                .map(|a| a.avg_quality())
                .sum();
            sum / r.agent_effectiveness.len() as f64
        };

        let score = (base * avg_quality).clamp(0.0, 1.0);

        MetricScore {
            metric: EvaluationMetric::Correctness,
            score,
            weight: EvaluationMetric::Correctness.default_weight(),
            details: format!(
                "Correctness: base={:.2}, avg_quality={:.2}",
                base, avg_quality
            ),
        }
    }

    /// Efficiency = inverse of retry ratio and normalized latency.
    pub fn compute_efficiency(&self, r: &MissionReflection) -> MetricScore {
        let retry_penalty = 1.0 - r.retry_ratio().min(1.0);
        let latency_factor = if r.total_tasks == 0 {
            1.0
        } else {
            let avg_latency = r.total_latency_ms as f64 / r.total_tasks as f64;
            // Normalize: <1000ms = excellent, >10000ms = poor
            (1.0 - (avg_latency / 10000.0).min(1.0)).max(0.0)
        };

        let score = (retry_penalty * 0.6 + latency_factor * 0.4).clamp(0.0, 1.0);

        MetricScore {
            metric: EvaluationMetric::Efficiency,
            score,
            weight: EvaluationMetric::Efficiency.default_weight(),
            details: format!(
                "retry_penalty={:.2}, latency_factor={:.2}",
                retry_penalty, latency_factor
            ),
        }
    }

    /// Cost efficiency = how well cost scales with complexity.
    pub fn compute_cost_efficiency(&self, r: &MissionReflection) -> MetricScore {
        // Normalize cost: <10 = excellent, >100 = poor
        let score = if r.total_cost <= 0.0 {
            1.0 // free is maximally cost-efficient
        } else {
            (1.0 - (r.total_cost / 100.0).min(1.0)).max(0.0)
        };

        MetricScore {
            metric: EvaluationMetric::CostEfficiency,
            score,
            weight: EvaluationMetric::CostEfficiency.default_weight(),
            details: format!("Total cost: {:.2}", r.total_cost),
        }
    }

    /// Safety = inverse failure rate.
    pub fn compute_safety(&self, r: &MissionReflection) -> MetricScore {
        let score = if r.total_tasks == 0 {
            1.0
        } else {
            1.0 - (r.failed_tasks as f64 / r.total_tasks as f64)
        };

        MetricScore {
            metric: EvaluationMetric::Safety,
            score,
            weight: EvaluationMetric::Safety.default_weight(),
            details: format!("{} failures out of {} tasks", r.failed_tasks, r.total_tasks),
        }
    }

    /// Confidence = composite from other metrics.
    pub fn compute_confidence(&self, r: &MissionReflection) -> MetricScore {
        let completeness = self.compute_completeness(r).score;
        let safety = self.compute_safety(r).score;
        let efficiency = self.compute_efficiency(r).score;

        // Confidence is the geometric mean of core metrics
        let score = (completeness * safety * efficiency)
            .powf(1.0 / 3.0)
            .clamp(0.0, 1.0);

        MetricScore {
            metric: EvaluationMetric::Confidence,
            score,
            weight: EvaluationMetric::Confidence.default_weight(),
            details: format!(
                "Derived from completeness={:.2}, safety={:.2}, efficiency={:.2}",
                completeness, safety, efficiency
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::models::MissionGrade;
    use crate::models::{AgentId, MissionId};
    use crate::reflection::mission_reflection::{AgentEffectivenessScore, MissionReflection};
    use std::collections::HashMap;

    fn make_reflection(
        total: u32,
        completed: u32,
        failed: u32,
        retries: u32,
        cost: f64,
        latency_ms: u64,
    ) -> MissionReflection {
        MissionReflection {
            mission_id: MissionId::new(),
            total_tasks: total,
            completed_tasks: completed,
            failed_tasks: failed,
            retries,
            tool_usage: HashMap::new(),
            agent_effectiveness: HashMap::new(),
            total_cost: cost,
            total_latency_ms: latency_ms,
            started_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
        }
    }

    #[test]
    fn perfect_mission_scores_excellent() {
        let engine = SelfEvaluationEngine::new();
        let r = make_reflection(10, 10, 0, 0, 5.0, 1000);
        let score = engine.evaluate_mission(&r);

        assert!(score.overall_score > 0.85);
        assert!(score.grade >= MissionGrade::Good);
    }

    #[test]
    fn completely_failed_mission() {
        let engine = SelfEvaluationEngine::new();
        let r = make_reflection(10, 0, 10, 5, 100.0, 50000);
        let score = engine.evaluate_mission(&r);

        assert!(score.overall_score < 0.4);
        assert!(score.grade <= MissionGrade::Poor);
    }

    #[test]
    fn partial_success_mission() {
        let engine = SelfEvaluationEngine::new();
        let r = make_reflection(10, 7, 3, 2, 30.0, 5000);
        let score = engine.evaluate_mission(&r);

        assert!(score.overall_score > 0.3);
        assert!(score.overall_score < 0.9);
    }

    #[test]
    fn empty_mission_scores_zero_completeness() {
        let engine = SelfEvaluationEngine::new();
        let r = make_reflection(0, 0, 0, 0, 0.0, 0);
        let completeness = engine.compute_completeness(&r);
        assert!((completeness.score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn full_completeness() {
        let engine = SelfEvaluationEngine::new();
        let r = make_reflection(5, 5, 0, 0, 0.0, 0);
        let c = engine.compute_completeness(&r);
        assert!((c.score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn safety_no_failures() {
        let engine = SelfEvaluationEngine::new();
        let r = make_reflection(10, 10, 0, 0, 0.0, 0);
        let s = engine.compute_safety(&r);
        assert!((s.score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn safety_all_failures() {
        let engine = SelfEvaluationEngine::new();
        let r = make_reflection(10, 0, 10, 0, 0.0, 0);
        let s = engine.compute_safety(&r);
        assert!((s.score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cost_efficiency_free() {
        let engine = SelfEvaluationEngine::new();
        let r = make_reflection(5, 5, 0, 0, 0.0, 0);
        let c = engine.compute_cost_efficiency(&r);
        assert!((c.score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cost_efficiency_expensive() {
        let engine = SelfEvaluationEngine::new();
        let r = make_reflection(5, 5, 0, 0, 100.0, 0);
        let c = engine.compute_cost_efficiency(&r);
        assert!((c.score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn efficiency_no_retries() {
        let engine = SelfEvaluationEngine::new();
        let r = make_reflection(5, 5, 0, 0, 0.0, 2500);
        let e = engine.compute_efficiency(&r);
        // retry_penalty = 1.0, latency_factor depends on avg
        assert!(e.score > 0.8);
    }

    #[test]
    fn efficiency_high_retries() {
        let engine = SelfEvaluationEngine::new();
        let r = make_reflection(5, 5, 0, 10, 0.0, 2500);
        let e = engine.compute_efficiency(&r);
        assert!(e.score < 0.5);
    }

    #[test]
    fn correctness_with_agent_quality() {
        let engine = SelfEvaluationEngine::new();
        let mut r = make_reflection(10, 10, 0, 0, 0.0, 0);
        let agent = AgentId::new();
        r.agent_effectiveness.insert(
            agent,
            AgentEffectivenessScore {
                tasks_completed: 10,
                tasks_failed: 0,
                total_quality: 9.0,
                total_latency_ms: 0,
                task_count: 10,
            },
        );
        let c = engine.compute_correctness(&r);
        assert!(c.score > 0.8);
    }

    #[test]
    fn confidence_is_bounded() {
        let engine = SelfEvaluationEngine::new();
        let r = make_reflection(10, 10, 0, 0, 0.0, 1000);
        let conf = engine.compute_confidence(&r);
        assert!(conf.score >= 0.0);
        assert!(conf.score <= 1.0);
    }

    #[test]
    fn all_six_metrics_present() {
        let engine = SelfEvaluationEngine::new();
        let r = make_reflection(5, 3, 2, 1, 20.0, 3000);
        let score = engine.evaluate_mission(&r);
        assert_eq!(score.metric_scores.len(), 6);
    }

    #[test]
    fn overall_score_bounded() {
        let engine = SelfEvaluationEngine::new();
        for total in [0, 1, 5, 10, 50] {
            for completed in 0..=total {
                let failed = total - completed;
                let r = make_reflection(total, completed, failed, 0, 10.0, 1000);
                let score = engine.evaluate_mission(&r);
                assert!(
                    score.overall_score >= 0.0 && score.overall_score <= 1.0,
                    "Score out of bounds: {} for total={}, completed={}",
                    score.overall_score,
                    total,
                    completed
                );
            }
        }
    }

    #[test]
    fn grade_matches_overall_score() {
        let engine = SelfEvaluationEngine::new();
        let r = make_reflection(10, 10, 0, 0, 0.0, 500);
        let score = engine.evaluate_mission(&r);
        assert_eq!(score.grade, grade_from_score(score.overall_score));
    }

    #[test]
    fn default_trait() {
        let engine = SelfEvaluationEngine::new();
        let r = make_reflection(1, 1, 0, 0, 0.0, 0);
        let score = engine.evaluate_mission(&r);
        assert!(score.overall_score >= 0.0);
    }

    #[test]
    fn metric_details_non_empty() {
        let engine = SelfEvaluationEngine::new();
        let r = make_reflection(5, 3, 2, 1, 20.0, 3000);
        let score = engine.evaluate_mission(&r);
        for ms in &score.metric_scores {
            assert!(!ms.details.is_empty());
        }
    }

    #[test]
    fn higher_completion_higher_score() {
        let engine = SelfEvaluationEngine::new();
        let r_low = make_reflection(10, 3, 7, 0, 10.0, 1000);
        let r_high = make_reflection(10, 9, 1, 0, 10.0, 1000);
        let s_low = engine.evaluate_mission(&r_low);
        let s_high = engine.evaluate_mission(&r_high);
        assert!(s_high.overall_score > s_low.overall_score);
    }
}
