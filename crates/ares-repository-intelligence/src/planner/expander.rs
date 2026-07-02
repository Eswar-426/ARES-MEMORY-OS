use crate::core::capabilities::Capability;
use crate::planner::intent::Intent;

/// Expands the minimal set of capabilities derived from intent
/// into the full set required for a comprehensive answer.
///
/// Example: Intent::ExplainEntity produces [WhyExists],
/// Expander adds [GraphSearch, GitHistory] as required dependencies.
pub struct CapabilityExpander;

impl CapabilityExpander {
    #[tracing::instrument(name = "CapabilityExpander::expand", skip(capabilities))]
    pub fn expand(intent: &Intent, capabilities: Vec<Capability>) -> Vec<Capability> {
        let start = std::time::Instant::now();
        let mut expanded = capabilities;

        match intent {
            Intent::ExplainEntity => {
                Self::ensure(&mut expanded, Capability::GraphSearch);
                Self::ensure(&mut expanded, Capability::GitHistory);
            }
            Intent::AnalyzeImpact => {
                Self::ensure(&mut expanded, Capability::GraphSearch);
            }
            Intent::FindPath => {
                // GraphSearch is already the primary capability
            }
            Intent::Dashboard => {
                Self::ensure(&mut expanded, Capability::Workspace);
            }
            Intent::Traceability => {
                Self::ensure(&mut expanded, Capability::GraphSearch);
                Self::ensure(&mut expanded, Capability::GitHistory);
                Self::ensure(&mut expanded, Capability::Requirements);
            }
            Intent::GeneralQuestion | Intent::Unknown => {}
        }

        let added_count = expanded.len();
        tracing::debug!(
            duration_ms = start.elapsed().as_millis(),
            original = added_count,
            expanded = expanded.len(),
            "Capabilities expanded"
        );
        expanded
    }

    fn ensure(caps: &mut Vec<Capability>, cap: Capability) {
        if !caps.contains(&cap) {
            caps.push(cap);
        }
    }
}
