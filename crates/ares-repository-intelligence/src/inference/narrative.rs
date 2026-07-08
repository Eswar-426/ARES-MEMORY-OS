use super::intent::ExtractedIntent;
use crate::models::{ConfidenceResult, ContributorRef, EngineeringEvidence, EvidenceItem};

/// Shared evidence flattener — only includes items NOT already rendered
/// in the narrative sections (folders, commits, contributors are skipped).
pub fn flatten_evidence(evidence: &EngineeringEvidence) -> Vec<EvidenceItem> {
    let mut items = Vec::new();

    // Dependencies — include with relationship type for quick scanning
    for dep in &evidence.dependents {
        if dep.is_test {
            continue;
        } // test deps shown separately
        items.push(EvidenceItem {
            category: format!("dependent ({})", dep.relationship),
            value: dep.label.clone(),
        });
    }
    for dep in &evidence.dependencies {
        if dep.is_test {
            continue;
        }
        items.push(EvidenceItem {
            category: format!("dependency ({})", dep.relationship),
            value: dep.label.clone(),
        });
    }

    // Test dependencies — grouped separately
    let test_deps: Vec<_> = evidence
        .dependencies
        .iter()
        .filter(|d| d.is_test)
        .chain(evidence.dependents.iter().filter(|d| d.is_test))
        .collect();
    if !test_deps.is_empty() {
        for td in test_deps {
            items.push(EvidenceItem {
                category: "test_dependency".to_string(),
                value: td.label.clone(),
            });
        }
    }

    // These are NOT in most narrative sections
    for req in &evidence.requirements {
        items.push(EvidenceItem {
            category: "requirement".into(),
            value: req.clone(),
        });
    }
    for dec in &evidence.decisions {
        items.push(EvidenceItem {
            category: "decision".into(),
            value: dec.clone(),
        });
    }
    for test in &evidence.tests {
        items.push(EvidenceItem {
            category: "test".into(),
            value: test.clone(),
        });
    }
    for doc in &evidence.documentation {
        items.push(EvidenceItem {
            category: "documentation".into(),
            value: doc.clone(),
        });
    }

    items
}

/// Check if the queried entity is a test file.
pub fn is_test_entity(evidence: &EngineeringEvidence) -> bool {
    let name = evidence
        .file_path
        .as_deref()
        .unwrap_or(&evidence.entity_label);
    name.contains("/tests/")
        || name.contains("\\tests\\")
        || name.ends_with("_test.py")
        || name.ends_with("_test.rs")
        || name.ends_with("_test.ts")
        || name.ends_with("_test.js")
        || name.starts_with("test_")
        || evidence.entity_label.starts_with("test_")
}

/// **Purpose** — derived from creation commit + folder context.
pub fn purpose_section(evidence: &EngineeringEvidence, intent: &ExtractedIntent) -> Option<String> {
    let mut lines = Vec::new();

    if is_test_entity(evidence) {
        lines.push(format!("`{}` is a **test file**.", evidence.entity_label));
    }

    if let Some(reason) = &intent.creation_reason {
        lines.push(format!(
            "`{}` was introduced to: \"{}\"",
            evidence.entity_label, reason
        ));
    }

    if !evidence.folders.is_empty() {
        let names: Vec<&str> = evidence.folders.iter().map(|f| f.label.as_str()).collect();
        lines.push(format!(
            "Located in `{}/`, suggesting a shared utility role.",
            names.join("/")
        ));
    }

    if lines.is_empty() {
        return None;
    }

    Some(format!("**Purpose**\n{}", lines.join("\n\n")))
}

/// **History** — creation + evolution timeline.
pub fn history_section(evidence: &EngineeringEvidence, intent: &ExtractedIntent) -> Option<String> {
    if evidence.commits.is_empty() {
        return None;
    }

    let mut lines = Vec::new();

    lines.push(format!(
        "• Created: \"{}\" (`{}`) by {}",
        intent.creation_reason.as_deref().unwrap_or("(no message)"),
        intent.creation_hash,
        intent.creation_author,
    ));

    for step in &intent.evolution {
        lines.push(format!(
            "• Modified: \"{}\" (`{}`) by {}",
            step.description, step.hash, step.author,
        ));
    }

    Some(format!(
        "**History** ({} commit{})\n{}",
        evidence.commits.len(),
        if evidence.commits.len() == 1 { "" } else { "s" },
        lines.join("\n"),
    ))
}

/// **Architecture** — hierarchy + dependencies + dependents.
pub fn architecture_section(evidence: &EngineeringEvidence) -> Option<String> {
    let mut lines = Vec::new();

    if !evidence.folders.is_empty() {
        let names: Vec<&str> = evidence.folders.iter().map(|f| f.label.as_str()).collect();
        lines.push(format!("Located in `{}/`.", names.join("/")));
    }

    if let Some(ref mod_) = evidence.parent_module {
        lines.push(format!("Part of module `{}`.", mod_.label));
    }

    if !evidence.dependents.is_empty() {
        let real_count = evidence.dependents.iter().filter(|d| !d.is_test).count();
        let real: Vec<&str> = evidence
            .dependents
            .iter()
            .filter(|d| !d.is_test)
            .take(5)
            .map(|d| d.label.as_str())
            .collect();
        let test_count = evidence.dependents.iter().filter(|d| d.is_test).count();
        if real_count > 0 {
            let more = if real_count > 5 {
                format!(" (+{} more)", real_count - 5)
            } else {
                "".to_string()
            };
            lines.push(format!(
                "Depended on by {} entities{}: {}",
                real_count,
                more,
                real.join(", ")
            ));
        }
        if test_count > 0 {
            lines.push(format!("Test dependents: {}", test_count));
        }
    } else if !lines.is_empty() {
        lines.push("No downstream code dependencies detected.".to_string());
    }

    if !evidence.dependencies.is_empty() {
        let real_count = evidence.dependencies.iter().filter(|d| !d.is_test).count();
        let real: Vec<&str> = evidence
            .dependencies
            .iter()
            .filter(|d| !d.is_test)
            .take(5)
            .map(|d| d.label.as_str())
            .collect();
        let test_count = evidence.dependencies.iter().filter(|d| d.is_test).count();
        if real_count > 0 {
            let more = if real_count > 5 {
                format!(" (+{} more)", real_count - 5)
            } else {
                "".to_string()
            };
            lines.push(format!(
                "Depends on {} entities{}: {}",
                real_count,
                more,
                real.join(", ")
            ));
        }
        if test_count > 0 {
            lines.push(format!("Test dependencies: {}", test_count));
        }
    }

    if lines.is_empty() {
        return None;
    }

    Some(format!("**Architecture**\n{}", lines.join("\n")))
}

/// **Ownership** — owners + contributors with commit percentages.
pub fn ownership_section(evidence: &EngineeringEvidence) -> Option<String> {
    let mut lines = Vec::new();

    if !evidence.owners.is_empty() {
        lines.push(format!(
            "Registered owner{}: {}",
            if evidence.owners.len() == 1 { "" } else { "s" },
            evidence.owners.join(", "),
        ));
    }

    if !evidence.contributors.is_empty() {
        let primary: Vec<&ContributorRef> = evidence
            .contributors
            .iter()
            .filter(|c| c.is_primary)
            .collect();
        let others: Vec<&ContributorRef> = evidence
            .contributors
            .iter()
            .filter(|c| !c.is_primary)
            .collect();

        if let Some(p) = primary.first() {
            let total = evidence.commits.len();
            let pct = (p.commit_count * 100).checked_div(total).unwrap_or(0);
            lines.push(format!(
                "Primary contributor: {} ({} commits, ~{}%)",
                p.name, p.commit_count, pct
            ));
        }
        for o in &others {
            lines.push(format!(
                "Contributor: {} ({} commits)",
                o.name, o.commit_count
            ));
        }
    }

    if lines.is_empty() {
        return None;
    }

    Some(format!("**Ownership**\n{}", lines.join("\n")))
}

/// **Confidence** — score with checkmark reasons.
pub fn confidence_section(confidence: &ConfidenceResult) -> String {
    let mut lines = vec![format!("**Confidence: {}%**", confidence.score)];

    for reason in &confidence.reasons {
        lines.push(format!("✓ {}", reason));
    }

    lines.join("\n")
}
