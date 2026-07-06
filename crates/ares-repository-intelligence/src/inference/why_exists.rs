use super::deterministic::DeterministicInference;
use super::intent::IntentExtractor;
use super::narrative;
use crate::models::{
    EngineeringEvidence, EngineeringInsight, InferenceMode, InsightMetadata,
};
use crate::services::confidence_engine::ConfidenceEngine;

pub struct WhyExistsGenerator;

impl DeterministicInference for WhyExistsGenerator {
    fn generate(&self, evidence: &EngineeringEvidence) -> EngineeringInsight {
        let start = std::time::Instant::now();
        let intent = IntentExtractor::extract(&evidence.commits, evidence.timestamps.as_ref());
        let confidence = ConfidenceEngine::calculate(evidence);

        let mut sections = Vec::new();

        if let Some(s) = narrative::purpose_section(evidence, &intent) {
            sections.push(s);
        }
        if let Some(s) = narrative::history_section(evidence, &intent) {
            sections.push(s);
        }
        if let Some(s) = narrative::architecture_section(evidence) {
            sections.push(s);
        }
        if let Some(s) = narrative::ownership_section(evidence) {
            sections.push(s);
        }

        if sections.is_empty() {
            sections.push(
                "⚠️ This entity has no known relationships. It may be orphaned or newly added."
                    .to_string(),
            );
        }

        let answer = sections.join("\n\n");
        let summary = build_summary_line(evidence, &intent);

        EngineeringInsight {
            answer,
            summary,
            confidence,
            evidence: narrative::flatten_evidence(evidence),
            recommendations: build_recommendations(evidence),
            warnings: build_warnings(evidence),
            mode: InferenceMode::Offline,
            metadata: InsightMetadata {
                duration_ms: start.elapsed().as_millis() as u64,
                evidence_sources: {
                    let mut srcs = vec!["graph".to_string()];
                    if !evidence.commits.is_empty() {
                        srcs.push("git".to_string());
                    }
                    srcs
                },
                generator: "WhyExistsGenerator".to_string(),
            },
        }
    }
}

fn build_summary_line(
    evidence: &EngineeringEvidence,
    intent: &super::intent::ExtractedIntent,
) -> String {
    let mut parts = Vec::new();
    if let Some(reason) = &intent.creation_reason {
        parts.push(format!("Introduced for: \"{}\"", reason));
    } else {
        parts.push(format!(
            "{} ({})",
            evidence.entity_label, evidence.entity_type
        ));
    }
    if !evidence.contributors.is_empty() {
        parts.push(format!(
            "{} contributor{}",
            evidence.contributors.len(),
            if evidence.contributors.len() == 1 {
                ""
            } else {
                "s"
            }
        ));
    }
    parts.join(". ")
}

fn build_warnings(evidence: &EngineeringEvidence) -> Vec<String> {
    let mut w = Vec::new();
    if evidence.dependents.is_empty()
        && evidence.dependencies.is_empty()
        && evidence.folders.is_empty()
    {
        w.push("Entity has no structural relationships — possible orphan.".to_string());
    }
    if evidence.owners.is_empty() {
        w.push("No registered owners.".to_string());
    }
    if evidence.requirements.is_empty() {
        w.push("No linked requirements — traceability gap.".to_string());
    }
    if evidence.commits.is_empty() {
        w.push("No git history available.".to_string());
    }
    if evidence.entity_type == "unknown" {
        w.push("Entity not found in knowledge graph.".to_string());
    }
    w
}

fn build_recommendations(evidence: &EngineeringEvidence) -> Vec<String> {
    let mut r = Vec::new();
    if evidence.owners.is_empty() {
        r.push("Assign an owner to improve accountability.".to_string());
    }
    if evidence.requirements.is_empty() {
        r.push("Link to a requirement for better traceability.".to_string());
    }
    if evidence.dependents.is_empty() && evidence.entity_type != "project" {
        r.push("Verify this entity is not orphaned.".to_string());
    }
    r
}
