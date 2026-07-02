use super::deterministic::DeterministicInference;
use super::intent::IntentExtractor;
use super::narrative;
use crate::models::{
    EngineeringEvidence, EngineeringInsight, InferenceMode, InsightMetadata,
};
use crate::services::confidence_engine::ConfidenceEngine;

pub struct ImpactGenerator;

impl DeterministicInference for ImpactGenerator {
    fn generate(&self, evidence: &EngineeringEvidence) -> EngineeringInsight {
        let start = std::time::Instant::now();
        let _intent = IntentExtractor::extract(&evidence.commits);
        let confidence = ConfidenceEngine::calculate(evidence);
        let mut sections = Vec::new();

        // ── Blast Radius ─────────────────────────────────────────
        let deps = &evidence.dependents;
        let real_deps: Vec<_> = deps.iter().filter(|d| !d.is_test).collect();
        if real_deps.is_empty() {
            sections.push(
                "**Blast Radius**\n✅ No immediate impact. This entity has no dependents and can be safely removed.".to_string(),
            );
        } else {
            let mut lines = vec![format!(
                "Removing `{}` would directly affect **{} modules**:",
                evidence.entity_label,
                real_deps.len(),
            )];
            let mut by_rel: std::collections::HashMap<String, Vec<&str>> =
                std::collections::HashMap::new();
            for dep in real_deps {
                by_rel
                    .entry(dep.relationship.clone())
                    .or_default()
                    .push(&dep.label);
            }
            for (rel, labels) in &by_rel {
                let listed = labels
                    .iter()
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("`, `");
                let suffix = if labels.len() > 5 {
                    format!(" +{} more", labels.len() - 5)
                } else {
                    String::new()
                };
                lines.push(format!("  • **{}**: `{}`{}", rel, listed, suffix));
            }
            sections.push(format!("**Blast Radius**\n{}", lines.join("\n")));
        }

        // ── Dependencies ─────────────────────────────────────────
        if !evidence.dependencies.is_empty() {
            let top: Vec<&str> = evidence
                .dependencies
                .iter()
                .take(5)
                .map(|d| d.label.as_str())
                .collect();
            let suffix = if evidence.dependencies.len() > 5 {
                format!(" +{}", evidence.dependencies.len() - 5)
            } else {
                String::new()
            };
            sections.push(format!(
                "**Dependencies**\n`{}` depends on **{} components**: `{}`{}",
                evidence.entity_label,
                evidence.dependencies.len(),
                top.join("`, `"),
                suffix,
            ));
        }

        // ── Change Velocity ──────────────────────────────────────
        if !evidence.commits.is_empty() {
            let mut lines = vec![format!(
                "{} commits across {} contributor{}.",
                evidence.commits.len(),
                evidence.contributors.len(),
                if evidence.contributors.len() == 1 {
                    ""
                } else {
                    "s"
                }
            )];
            if evidence.contributors.len() > 3 {
                lines.push(
                    "⚡ High contributor count — coordination overhead if broken.".to_string(),
                );
            } else if evidence.contributors.len() == 1 && evidence.commits.len() > 10 {
                lines.push(
                    "⚠️ Single contributor with many commits — knowledge concentration risk."
                        .to_string(),
                );
            }
            sections.push(format!("**Change Velocity**\n{}", lines.join("\n")));
        }

        // ── Risk ─────────────────────────────────────────────────
        let dep_count = deps.len();
        let risk = if dep_count == 0 {
            "LOW"
        } else if dep_count <= 3 {
            "MEDIUM"
        } else {
            "HIGH"
        };
        sections.push(format!("**Risk Level: {}**", risk));

        // ── Confidence ───────────────────────────────────────────

        let answer = sections.join("\n\n");

        EngineeringInsight {
            answer,
            summary: format!(
                "Removing `{}` would affect {} modules. Risk: {}",
                evidence.entity_label, dep_count, risk
            ),
            confidence,
            evidence: narrative::flatten_evidence(evidence),
            warnings: if dep_count == 0 {
                vec![]
            } else {
                vec![format!("{} modules depend on this entity", dep_count)]
            },
            recommendations: if dep_count > 5 {
                vec!["Consider refactoring to reduce tight coupling.".to_string()]
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
                generator: "ImpactGenerator".to_string(),
            },
        }
    }
}
