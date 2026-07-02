use crate::models::{ConfidenceResult, EngineeringEvidence};

// ═══════════════════════════════════════════════════════════════════
// ConfidenceEngine — standalone, reusable scorer
// ═══════════════════════════════════════════════════════════════════

pub struct ConfidenceEngine;

impl ConfidenceEngine {
    pub fn calculate(evidence: &EngineeringEvidence) -> ConfidenceResult {
        let mut score: u8 = 0;
        let mut reasons = Vec::new();

        // Structural relationships
        if !evidence.dependents.is_empty() {
            score = score.saturating_add(25);
            reasons.push(format!("{} dependents", evidence.dependents.len()));
        }

        if !evidence.dependencies.is_empty() {
            score = score.saturating_add(20);
            reasons.push(format!("{} dependencies", evidence.dependencies.len()));
        }

        if !evidence.folders.is_empty() {
            score = score.saturating_add(10);
            reasons.push(format!(
                "Located in {} folder{}",
                evidence.folders.len(),
                if evidence.folders.len() == 1 { "" } else { "s" }
            ));
        }

        if evidence.parent_module.is_some() {
            score = score.saturating_add(10);
            reasons.push("Parent module identified".to_string());
        }

        // Git history
        if !evidence.commits.is_empty() {
            score = score.saturating_add(20);
            reasons.push(format!("{} git commits", evidence.commits.len()));
        }

        if !evidence.contributors.is_empty() {
            score = score.saturating_add(10);
            reasons.push(format!(
                "{} known contributors",
                evidence.contributors.len()
            ));
            if evidence.contributors.len() > 1 {
                score = score.saturating_add(10);
                reasons.push("Multiple contributors (reduced bus factor risk)".to_string());
            }
        }

        // Ownership
        if !evidence.owners.is_empty() {
            score = score.saturating_add(10);
            reasons.push(format!("{} registered owners", evidence.owners.len()));
        }

        // Architecture
        if !evidence.requirements.is_empty() {
            score = score.saturating_add(10);
            reasons.push(format!(
                "{} linked requirements",
                evidence.requirements.len()
            ));
        }

        if !evidence.decisions.is_empty() {
            score = score.saturating_add(5);
            reasons.push(format!("{} linked decisions", evidence.decisions.len()));
        }

        // Metrics
        if let Some(ref m) = evidence.metrics {
            let has_any = m.lines_of_code.is_some() || m.complexity.is_some();
            if has_any {
                score = score.saturating_add(5);
                reasons.push("Code metrics available".to_string());
            }
        }

        if evidence.timestamps.is_some() {
            score = score.saturating_add(5);
            reasons.push("Timestamps available".to_string());
        }

        // Recency bonus — requires commit timestamps from scanner (not yet available)
        // evidence.timestamps.last_committed will be populated once ares-git-memory
        // extracts authored_date from git log.

        if reasons.is_empty() {
            reasons.push("No evidence sources found".to_string());
        }

        ConfidenceResult {
            score: score.min(100),
            reasons,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EngineeringEvidence;

    #[test]
    fn empty_evidence_scores_zero() {
        let evidence = EngineeringEvidence::not_found("test", "proj");
        let result = ConfidenceEngine::calculate(&evidence);
        assert_eq!(result.score, 0);
        assert!(!result.reasons.is_empty());
    }
}
