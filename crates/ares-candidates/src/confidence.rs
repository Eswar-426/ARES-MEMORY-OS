use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateConfidence {
    pub evidence_count: u32,
    pub source_diversity: u32,
    pub temporal_consistency: f64,
    pub cluster_strength: f64,
}

impl CandidateConfidence {
    pub fn overall_score(&self) -> f64 {
        let mut score = 0.0;

        // EvidenceCount (30%)
        score += (self.evidence_count as f64).min(50.0) / 50.0 * 30.0;

        // SourceDiversity (25%)
        score += (self.source_diversity as f64).min(10.0) / 10.0 * 25.0;

        // TemporalConsistency (20%)
        score += self.temporal_consistency.clamp(0.0, 1.0) * 20.0;

        // ClusterStrength (25%)
        score += self.cluster_strength.clamp(0.0, 1.0) * 25.0;

        score.clamp(0.0, 100.0)
    }

    pub fn normalized_score(&self) -> f64 {
        self.overall_score() / 100.0
    }
}

pub struct CandidateThresholds;

impl CandidateThresholds {
    pub fn requirement() -> f64 {
        0.65
    }

    pub fn decision() -> f64 {
        0.80
    }

    pub fn architecture() -> f64 {
        0.85
    }

    pub fn traceability() -> f64 {
        0.75
    }

    pub fn capability() -> f64 {
        0.85
    }

    pub fn ownership() -> f64 {
        0.90
    }

    pub fn for_type(candidate_type: &crate::models::CandidateType) -> f64 {
        match candidate_type {
            crate::models::CandidateType::Requirement => Self::requirement(),
            crate::models::CandidateType::Decision => Self::decision(),
            crate::models::CandidateType::Architecture => Self::architecture(),
            crate::models::CandidateType::Traceability => Self::traceability(),
            crate::models::CandidateType::Capability => Self::capability(),
            crate::models::CandidateType::Ownership => Self::ownership(),
        }
    }
    pub fn get_traceability_strength(score: f64) -> crate::models::TraceabilityStrength {
        if score >= 0.95 {
            crate::models::TraceabilityStrength::Definitive
        } else if score >= 0.90 {
            crate::models::TraceabilityStrength::Strong
        } else if score >= 0.80 {
            crate::models::TraceabilityStrength::Moderate
        } else {
            crate::models::TraceabilityStrength::Weak
        }
    }
}

impl From<f64> for CandidateConfidence {
    fn from(score: f64) -> Self {
        Self {
            evidence_count: 1,
            source_diversity: 1,
            temporal_consistency: score,
            cluster_strength: score,
        }
    }
}
