use crate::core::evidence::RawEvidenceBundle;

/// Validates invariants on the aggregated RawEvidenceBundle before
/// it enters the Knowledge Pipeline.
///
/// Invariants checked:
/// - Unique IDs across arrays
/// - Confidence in [0, 1]
/// - No duplicate paths/commits
/// - Normalized file paths (forward slashes)
pub struct EvidenceValidator;

#[derive(Debug, Default)]
pub struct ValidationResult {
    pub issues: Vec<String>,
    pub fixes_applied: usize,
}

impl EvidenceValidator {
    #[tracing::instrument(name = "EvidenceValidator::validate", skip(bundle))]
    pub fn validate(bundle: &mut RawEvidenceBundle) -> ValidationResult {
        let start = std::time::Instant::now();
        let mut result = ValidationResult::default();

        // 1. Deduplicate graph node IDs
        if let Some(ref mut graph) = bundle.graph {
            let before = graph.nodes.len();
            graph.nodes.sort();
            graph.nodes.dedup();
            if graph.nodes.len() < before {
                let removed = before - graph.nodes.len();
                result.fixes_applied += removed;
                result
                    .issues
                    .push(format!("Removed {} duplicate graph node IDs", removed));
            }

            let before = graph.edges.len();
            graph.edges.sort();
            graph.edges.dedup();
            if graph.edges.len() < before {
                let removed = before - graph.edges.len();
                result.fixes_applied += removed;
                result
                    .issues
                    .push(format!("Removed {} duplicate graph edge IDs", removed));
            }

            let before = graph.paths.len();
            graph.paths.sort();
            graph.paths.dedup();
            if graph.paths.len() < before {
                let removed = before - graph.paths.len();
                result.fixes_applied += removed;
                result
                    .issues
                    .push(format!("Removed {} duplicate graph paths", removed));
            }
        }

        // 2. Deduplicate git commits/authors
        if let Some(ref mut git) = bundle.git {
            let before = git.commits.len();
            git.commits.sort();
            git.commits.dedup();
            if git.commits.len() < before {
                let removed = before - git.commits.len();
                result.fixes_applied += removed;
                result
                    .issues
                    .push(format!("Removed {} duplicate commits", removed));
            }

            let before = git.authors.len();
            git.authors.sort();
            git.authors.dedup();
            if git.authors.len() < before {
                let removed = before - git.authors.len();
                result.fixes_applied += removed;
                result
                    .issues
                    .push(format!("Removed {} duplicate authors", removed));
            }
        }

        // 3. Normalize file paths in code evidence
        if let Some(ref mut code) = bundle.code {
            for file in code.files.iter_mut() {
                let normalized = file.replace('\\', "/");
                if normalized != *file {
                    result.fixes_applied += 1;
                    *file = normalized;
                }
            }
        }

        // 4. Clamp confidence
        if let Some(ref mut runtime) = bundle.runtime {
            if runtime.confidence < 0.0 {
                runtime.confidence = 0.0;
                result.fixes_applied += 1;
                result
                    .issues
                    .push("Clamped confidence from negative to 0.0".to_string());
            }
            if runtime.confidence > 1.0 {
                runtime.confidence = 1.0;
                result.fixes_applied += 1;
                result
                    .issues
                    .push("Clamped confidence above 1.0 to 1.0".to_string());
            }
        }

        tracing::debug!(
            duration_ms = start.elapsed().as_millis(),
            issues = result.issues.len(),
            fixes = result.fixes_applied,
            "Evidence validation complete"
        );
        result
    }
}
