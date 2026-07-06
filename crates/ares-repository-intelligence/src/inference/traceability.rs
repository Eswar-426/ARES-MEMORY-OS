use super::deterministic::DeterministicInference;
use super::intent::IntentExtractor;
use super::narrative;
use crate::models::{
    EngineeringEvidence, EngineeringInsight, InferenceMode, InsightMetadata,
};
use crate::services::confidence_engine::ConfidenceEngine;

pub struct TraceabilityGenerator;

impl DeterministicInference for TraceabilityGenerator {
    fn generate(&self, evidence: &EngineeringEvidence) -> EngineeringInsight {
        let start = std::time::Instant::now();
        let intent = IntentExtractor::extract(&evidence.commits, evidence.timestamps.as_ref());
        let confidence = ConfidenceEngine::calculate(evidence);
        let mut sections = Vec::new();

        // ── Origin ───────────────────────────────────────────────
        if let Some(reason) = &intent.creation_reason {
            sections.push(format!(
                "**Origin**\nIntroduced by {} in commit `{}`: \"{}\"",
                intent.creation_author, intent.creation_hash, reason
            ));
        } else if !evidence.commits.is_empty() {
            sections.push(format!(
                "**Origin**\nIntroduced in commit `{}`.",
                intent.creation_hash
            ));
        } else {
            sections.push("**Origin**\n⚠️ No git history available.".to_string());
        }

        // ── Evolution ────────────────────────────────────────────
        if !intent.evolution.is_empty() {
            let lines: Vec<String> = intent
                .evolution
                .iter()
                .map(|s| format!("• \"{}\" (`{}`) by {}", s.description, s.hash, s.author))
                .collect();
            sections.push(format!("**Evolution**\n{}", lines.join("\n")));
        }

        // ── Contributors ─────────────────────────────────────────
        if !evidence.contributors.is_empty() {
            let primary: Vec<&str> = evidence
                .contributors
                .iter()
                .filter(|c| c.is_primary)
                .map(|c| c.name.as_str())
                .collect();
            let others: Vec<&str> = evidence
                .contributors
                .iter()
                .filter(|c| !c.is_primary)
                .map(|c| c.name.as_str())
                .collect();
            let mut lines = Vec::new();
            if !primary.is_empty() {
                lines.push(format!("Primary: {}", primary.join(", ")));
            }
            if !others.is_empty() {
                lines.push(format!("Others: {}", others.join(", ")));
            }
            sections.push(format!("**Contributors**\n{}", lines.join("\n")));
        }

        // ── Requirements ─────────────────────────────────────────
        if evidence.requirements.is_empty() {
            sections.push("**Requirements**\n⚠️ None linked — traceability gap.".to_string());
        } else {
            let lines: Vec<String> = evidence
                .requirements
                .iter()
                .map(|r| format!("• {}", r))
                .collect();
            sections.push(format!(
                "**Requirements** ({} linked)\n{}",
                evidence.requirements.len(),
                lines.join("\n")
            ));
        }

        // ── Decisions ────────────────────────────────────────────
        if evidence.decisions.is_empty() {
            sections.push("**Architectural Decisions**\n⚠️ None linked.".to_string());
        } else {
            let lines: Vec<String> = evidence
                .decisions
                .iter()
                .map(|d| format!("• {}", d))
                .collect();
            sections.push(format!(
                "**Architectural Decisions** ({} linked)\n{}",
                evidence.decisions.len(),
                lines.join("\n")
            ));
        }

        // ── Ownership ────────────────────────────────────────────
        if evidence.owners.is_empty() {
            sections.push("**Ownership**\n⚠️ Unassigned — accountability gap.".to_string());
        } else {
            sections.push(format!("**Ownership**\n{}", evidence.owners.join(", ")));
        }

        // ── Confidence ───────────────────────────────────────────

        let answer = sections.join("\n\n");

        EngineeringInsight {
            answer,
            summary: format!(
                "{} has {} commits, {} requirements, {} owners.",
                evidence.entity_label,
                evidence.commits.len(),
                evidence.requirements.len(),
                evidence.owners.len(),
            ),
            confidence,
            evidence: narrative::flatten_evidence(evidence),
            warnings: {
                let mut w = Vec::new();
                if evidence.commits.is_empty() {
                    w.push("No git history".into());
                }
                if evidence.requirements.is_empty() {
                    w.push("No requirements linked".into());
                }
                if evidence.owners.is_empty() {
                    w.push("No owner assigned".into());
                }
                w
            },
            recommendations: if evidence.requirements.is_empty() {
                vec!["Link to a requirement for full traceability.".into()]
            } else {
                vec![]
            },
            mode: InferenceMode::Offline,
            metadata: InsightMetadata {
                duration_ms: start.elapsed().as_millis() as u64,
                evidence_sources: {
                    let mut s = vec!["graph".to_string()];
                    if !evidence.commits.is_empty() {
                        s.push("git".to_string());
                    }
                    s
                },
                generator: "TraceabilityGenerator".to_string(),
            },
        }
    }
}
