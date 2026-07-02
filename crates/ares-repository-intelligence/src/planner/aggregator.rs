use crate::core::engine::EngineExecutionResult;
use crate::core::evidence::EvidenceBundle;

pub struct EvidenceAggregator;

impl EvidenceAggregator {
    #[tracing::instrument(name = "EvidenceAggregator::aggregate", skip(results))]
    pub fn aggregate(results: Vec<EngineExecutionResult>) -> EvidenceBundle {
        let start = std::time::Instant::now();
        // Mock returning a default EvidenceBundle for now
        let bundle = EvidenceBundle::default();

        // In reality, we would merge results into bundle.
        // For now just tracking duration.

        tracing::debug!(
            duration_ms = start.elapsed().as_millis(),
            results_merged = results.len(),
            "Aggregated evidence"
        );
        bundle
    }
}
