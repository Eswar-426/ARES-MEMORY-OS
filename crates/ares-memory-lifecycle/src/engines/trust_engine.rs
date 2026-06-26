use crate::models::TrustScore;

pub struct TrustEngine {
    pub minimum_evidence: usize,
}

impl TrustEngine {
    pub fn new(minimum_evidence: usize) -> Self {
        Self { minimum_evidence }
    }

    pub fn evaluate_trust(
        &self,
        evidence_count: usize,
        manual_approvals: usize,
        revalidation_successes: usize,
        contradiction_signals: usize,
    ) -> TrustScore {
        let mut base_score = 0.0;

        if evidence_count >= self.minimum_evidence {
            base_score += 0.5;
        } else if evidence_count > 0 {
            base_score += 0.25;
        }

        if manual_approvals > 0 {
            base_score += 0.3;
        }

        if revalidation_successes > 0 {
            base_score += 0.2;
        }

        // Penalty
        let penalty = (contradiction_signals as f32) * 0.4;
        let mut score = base_score - penalty;

        score = score.clamp(0.0, 1.0);

        let is_trusted = score >= 0.5 && contradiction_signals == 0;

        TrustScore {
            score,
            evidence_count,
            manual_approvals,
            revalidation_successes,
            contradiction_signals,
            is_trusted,
        }
    }
}
