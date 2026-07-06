use super::deterministic::DeterministicInference;
use super::intent::IntentExtractor;
use super::narrative;
use crate::models::{
    EngineeringEvidence, EngineeringInsight, InferenceMode, InsightMetadata,
};
use crate::services::confidence_engine::ConfidenceEngine;

pub struct DriftGenerator;

impl DeterministicInference for DriftGenerator {
    fn generate(&self, evidence: &EngineeringEvidence) -> EngineeringInsight {
        let start = std::time::Instant::now();
        let _intent = IntentExtractor::extract(&evidence.commits, evidence.timestamps.as_ref());
        let confidence = ConfidenceEngine::calculate(evidence);
        let mut sections = Vec::new();
        let mut drift_score: u8 = 0;

        // ── Stability Assessment ─────────────────────────────────
        let commit_count = evidence.commits.len();
        if commit_count == 0 {
            sections.push(
                "**Stability**\n⚠️ No commit history — drift cannot be calculated.".to_string(),
            );
        } else {
            let mut lines = vec![format!("{} total commits.", commit_count)];
            if commit_count > 10 {
                drift_score += 40;
                lines.push(
                    "⚡ High modification frequency — may be unstable or actively evolving."
                        .to_string(),
                );
            } else if commit_count > 3 {
                drift_score += 20;
                lines.push("🔄 Moderate modification frequency — normal evolution.".to_string());
            } else {
                lines.push("✅ Low modification frequency — stable entity.".to_string());
            }
            sections.push(format!("**Stability**\n{}", lines.join("\n")));
        }

        // Staleness — requires commit timestamps from scanner (not yet available)

        // ── Contributor Analysis ─────────────────────────────────
        if !evidence.contributors.is_empty() {
            let mut lines = vec![format!(
                "{} contributor{}.",
                evidence.contributors.len(),
                if evidence.contributors.len() == 1 {
                    ""
                } else {
                    "s"
                }
            )];
            if evidence.contributors.len() == 1 && commit_count > 5 {
                drift_score += 15;
                lines.push(
                    "⚠️ Single author with significant history — bus factor risk.".to_string(),
                );
            } else if evidence.contributors.len() > 5 {
                drift_score += 10;
                lines.push("🔄 High contributor churn — possible instability.".to_string());
            } else {
                lines.push("✅ Healthy contributor distribution.".to_string());
            }
            sections.push(format!("**Contributor Analysis**\n{}", lines.join("\n")));
        }

        // ── Dependency Profile ───────────────────────────────────
        let dep_count = evidence.dependencies.len();
        let dependent_count = evidence.dependents.len();
        let real_dep_count = evidence.dependencies.iter().filter(|d| !d.is_test).count();
        let real_dependent_count = evidence.dependents.iter().filter(|d| !d.is_test).count();
        let mut dep_lines = vec![
            format!(
                "• Dependents: {} ({} tests excluded)",
                real_dependent_count,
                dependent_count - real_dependent_count
            ),
            format!(
                "• Dependencies: {} ({} tests excluded)",
                real_dep_count,
                dep_count - real_dep_count
            ),
        ];
        if real_dep_count > 10 {
            drift_score += 30;
            dep_lines.push("⚡ Heavy dependency footprint — high coupling risk.".to_string());
        } else if real_dep_count > 5 {
            drift_score += 15;
            dep_lines.push("⚠️ Moderate coupling.".to_string());
        }
        sections.push(format!("**Dependency Profile**\n{}", dep_lines.join("\n")));

        // ── Orphan Check ─────────────────────────────────────────
        if dependent_count == 0 && dep_count == 0 && evidence.folders.is_empty() {
            drift_score += 50;
            sections.push(
                "**Orphan Status**\n🚨 No structural connections — possible dead code.".to_string(),
            );
        }

        // ── Documentation ────────────────────────────────────────
        if evidence.documentation.is_empty() {
            drift_score += 15;
            sections.push(
                "**Documentation**\n⚠️ No documentation found — documentation drift likely."
                    .to_string(),
            );
        }

        // ── Verdict ──────────────────────────────────────────────
        let level = if drift_score >= 60 {
            "HIGH"
        } else if drift_score >= 30 {
            "MEDIUM"
        } else {
            "LOW"
        };
        sections.push(format!(
            "**Drift Verdict: {}** (score: {})",
            level, drift_score
        ));

        // ── Confidence ───────────────────────────────────────────

        let answer = sections.join("\n\n");

        EngineeringInsight {
            answer,
            summary: format!(
                "Drift: {} (score: {}). {} commits, {} dependents, {} dependencies.",
                level, drift_score, commit_count, dependent_count, dep_count
            ),
            confidence,
            evidence: narrative::flatten_evidence(evidence),
            warnings: {
                let mut w = Vec::new();
                if commit_count == 0 {
                    w.push("No commit history".into());
                }
                if evidence.documentation.is_empty() {
                    w.push("No documentation".into());
                }
                if dependent_count == 0 && dep_count == 0 && evidence.folders.is_empty() {
                    w.push("Orphan — no connections".into());
                }
                w
            },
            recommendations: if drift_score >= 60 {
                vec!["Review for potential refactoring or removal.".into()]
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
                generator: "DriftGenerator".to_string(),
            },
        }
    }
}
