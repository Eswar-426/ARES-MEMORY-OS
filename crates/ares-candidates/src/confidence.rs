use serde::{Serialize, Deserialize};

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
}
