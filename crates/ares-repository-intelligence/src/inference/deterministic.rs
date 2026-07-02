use crate::models::{EngineeringEvidence, EngineeringInsight};

// ═══════════════════════════════════════════════════════════════════
// DeterministicInference — the trait all generators implement
// ═══════════════════════════════════════════════════════════════════

/// A deterministic intelligence generator.
///
/// Every generator (WhyExists, Impact, Ownership, Drift, …)
/// implements this trait. The generator reads from
/// `EngineeringEvidence` (never the database) and produces an
/// `EngineeringInsight`.
///
/// Adding a new intelligence feature means:
/// 1. Create a struct implementing `DeterministicInference`
/// 2. Register it in the `InferenceRegistry`
///
/// Existing code never needs to change.
pub trait DeterministicInference: Send + Sync {
    /// Generate an insight from the collected evidence.
    fn generate(&self, evidence: &EngineeringEvidence) -> EngineeringInsight;
}
