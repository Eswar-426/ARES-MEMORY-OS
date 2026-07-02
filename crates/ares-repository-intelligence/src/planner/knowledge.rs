use crate::core::evidence::{ProcessedEvidenceBundle, RawEvidenceBundle};

/// The Knowledge Pipeline transforms validated raw evidence into
/// processed, enriched evidence. Structured as a pipeline of stages:
///
/// RawEvidenceBundle
///     → Canonicalizer
///     → Deduplicator
///     → ConfidenceCalculator
///     → RelationshipResolver
///     → ContextRanker
/// ProcessedEvidenceBundle
pub struct KnowledgePipeline;

impl KnowledgePipeline {
    #[tracing::instrument(name = "KnowledgePipeline::process", skip(raw))]
    pub fn process(raw: RawEvidenceBundle) -> ProcessedEvidenceBundle {
        let start = std::time::Instant::now();

        // Stage 1: Convert raw to processed (base conversion)
        let mut processed: ProcessedEvidenceBundle = raw.into();

        // Stage 2: Canonicalize — collect all unique IDs
        let mut canonical_ids = Vec::new();
        if let Some(ref graph) = processed.graph {
            canonical_ids.extend(graph.nodes.clone());
        }
        if let Some(ref git) = processed.git {
            canonical_ids.extend(git.commits.clone());
        }
        canonical_ids.sort();
        canonical_ids.dedup();
        processed.canonical_ids = canonical_ids;

        // Stage 3: Deduplication flag
        processed.deduplicated = true;

        // Stage 4: Confidence calculation
        let mut total_confidence = 0.0_f32;
        let mut source_count = 0;
        if processed.graph.is_some() {
            total_confidence += 1.0;
            source_count += 1;
        }
        if processed.git.is_some() {
            total_confidence += 1.0;
            source_count += 1;
        }
        if processed.architecture.is_some() {
            total_confidence += 1.0;
            source_count += 1;
        }
        if processed.code.is_some() {
            total_confidence += 0.8;
            source_count += 1;
        }
        if let Some(ref runtime) = processed.runtime {
            total_confidence += runtime.confidence;
            source_count += 1;
        }
        processed.confidence = if source_count > 0 {
            total_confidence / source_count as f32
        } else {
            0.0
        };

        // Stage 5: RelationshipResolver — placeholder for future cross-evidence linking
        // Stage 6: ContextRanker — placeholder for future relevance scoring

        tracing::debug!(
            duration_ms = start.elapsed().as_millis(),
            canonical_count = processed.canonical_ids.len(),
            confidence = processed.confidence,
            "Knowledge pipeline complete"
        );
        processed
    }
}
