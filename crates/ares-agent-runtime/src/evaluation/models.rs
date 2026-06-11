use crate::models::MissionId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Evaluation dimensions for scoring a mission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvaluationMetric {
    Completeness,
    Correctness,
    Efficiency,
    CostEfficiency,
    Safety,
    Confidence,
}

impl EvaluationMetric {
    /// Default weight for this metric in the overall score.
    pub fn default_weight(&self) -> f64 {
        match self {
            Self::Completeness => 0.25,
            Self::Correctness => 0.25,
            Self::Efficiency => 0.15,
            Self::CostEfficiency => 0.10,
            Self::Safety => 0.15,
            Self::Confidence => 0.10,
        }
    }
}

/// Score for a single evaluation metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricScore {
    pub metric: EvaluationMetric,
    /// Score in [0.0, 1.0].
    pub score: f64,
    /// Weight used in overall computation.
    pub weight: f64,
    /// Human-readable explanation.
    pub details: String,
}

/// Overall mission quality grade derived from score thresholds.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MissionGrade {
    Failed,
    Poor,
    Acceptable,
    Good,
    Excellent,
}

/// Composite mission evaluation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionScore {
    pub mission_id: MissionId,
    /// Weighted average of all metric scores, in [0.0, 1.0].
    pub overall_score: f64,
    pub metric_scores: Vec<MetricScore>,
    pub evaluated_at: DateTime<Utc>,
    pub grade: MissionGrade,
}

/// Derive a grade from a numeric score.
pub fn grade_from_score(score: f64) -> MissionGrade {
    if score >= 0.9 {
        MissionGrade::Excellent
    } else if score >= 0.75 {
        MissionGrade::Good
    } else if score >= 0.6 {
        MissionGrade::Acceptable
    } else if score >= 0.4 {
        MissionGrade::Poor
    } else {
        MissionGrade::Failed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grade_thresholds() {
        assert_eq!(grade_from_score(0.95), MissionGrade::Excellent);
        assert_eq!(grade_from_score(0.90), MissionGrade::Excellent);
        assert_eq!(grade_from_score(0.80), MissionGrade::Good);
        assert_eq!(grade_from_score(0.75), MissionGrade::Good);
        assert_eq!(grade_from_score(0.65), MissionGrade::Acceptable);
        assert_eq!(grade_from_score(0.60), MissionGrade::Acceptable);
        assert_eq!(grade_from_score(0.50), MissionGrade::Poor);
        assert_eq!(grade_from_score(0.40), MissionGrade::Poor);
        assert_eq!(grade_from_score(0.30), MissionGrade::Failed);
        assert_eq!(grade_from_score(0.0), MissionGrade::Failed);
    }

    #[test]
    fn grade_ordering() {
        assert!(MissionGrade::Failed < MissionGrade::Poor);
        assert!(MissionGrade::Poor < MissionGrade::Acceptable);
        assert!(MissionGrade::Acceptable < MissionGrade::Good);
        assert!(MissionGrade::Good < MissionGrade::Excellent);
    }

    #[test]
    fn metric_weights_sum_to_one() {
        let metrics = [
            EvaluationMetric::Completeness,
            EvaluationMetric::Correctness,
            EvaluationMetric::Efficiency,
            EvaluationMetric::CostEfficiency,
            EvaluationMetric::Safety,
            EvaluationMetric::Confidence,
        ];
        let sum: f64 = metrics.iter().map(|m| m.default_weight()).sum();
        assert!((sum - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn metric_score_serialization() {
        let ms = MetricScore {
            metric: EvaluationMetric::Safety,
            score: 0.85,
            weight: 0.15,
            details: "Low failure rate".to_string(),
        };
        let json = serde_json::to_string(&ms).unwrap();
        let back: MetricScore = serde_json::from_str(&json).unwrap();
        assert_eq!(back.metric, EvaluationMetric::Safety);
        assert!((back.score - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn mission_score_serialization() {
        let ms = MissionScore {
            mission_id: MissionId::new(),
            overall_score: 0.82,
            metric_scores: vec![],
            evaluated_at: Utc::now(),
            grade: MissionGrade::Good,
        };
        let json = serde_json::to_string(&ms).unwrap();
        let back: MissionScore = serde_json::from_str(&json).unwrap();
        assert_eq!(back.grade, MissionGrade::Good);
    }
}
