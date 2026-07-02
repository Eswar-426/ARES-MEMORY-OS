use crate::core::capabilities::Capability;
use crate::planner::intent::Intent;

/// Removes unnecessary capabilities from the expanded set.
/// Runs after CapabilityExpander to prune redundant or irrelevant engines.
pub struct CapabilityOptimizer;

impl CapabilityOptimizer {
    #[tracing::instrument(name = "CapabilityOptimizer::optimize", skip(capabilities))]
    pub fn optimize(
        intent: &Intent,
        capabilities: Vec<Capability>,
    ) -> (Vec<Capability>, Vec<String>) {
        let start = std::time::Instant::now();
        let original_count = capabilities.len();
        let mut optimized = capabilities;
        let mut removed = Vec::new();

        match intent {
            Intent::ExplainEntity => {
                // Impact is not needed for "why does X exist?"
                if let Some(pos) = optimized
                    .iter()
                    .position(|c| *c == Capability::ImpactAnalysis)
                {
                    removed.push(format!("{:?}", optimized.remove(pos)));
                }
                if let Some(pos) = optimized.iter().position(|c| *c == Capability::Simulation) {
                    removed.push(format!("{:?}", optimized.remove(pos)));
                }
            }
            Intent::AnalyzeImpact => {
                // Don't need full requirements or traceability for blast radius
                if let Some(pos) = optimized
                    .iter()
                    .position(|c| *c == Capability::Requirements)
                {
                    removed.push(format!("{:?}", optimized.remove(pos)));
                }
            }
            Intent::Dashboard => {
                // Dashboard only needs workspace; prune everything else
                optimized.retain(|c| *c == Capability::Workspace);
            }
            _ => {}
        }

        tracing::debug!(
            duration_ms = start.elapsed().as_millis(),
            original = original_count,
            final_count = optimized.len(),
            removed = ?removed,
            "Capabilities optimized"
        );
        (optimized, removed)
    }
}
