use super::deterministic::DeterministicInference;
use super::why_exists::WhyExistsGenerator;
use crate::models::{EngineeringEvidence, EngineeringInsight, QueryType};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════
// InferenceRegistry — extensible dispatch without match
// ═══════════════════════════════════════════════════════════════════

/// Registry of deterministic inference generators.
///
/// Instead of a `match` statement that must be modified for every
/// new feature, generators register themselves by `QueryType` key.
/// Adding a new intelligence feature never modifies existing code.
pub struct InferenceRegistry {
    generators: HashMap<String, Box<dyn DeterministicInference>>,
}

impl InferenceRegistry {
    /// Create a new registry pre-loaded with all built-in generators.
    pub fn new() -> Self {
        let mut registry = Self {
            generators: HashMap::new(),
        };

        // Register built-in generators
        registry.register("why_exists", Box::new(WhyExistsGenerator));
        registry.register(
            "impact",
            Box::new(crate::inference::impact::ImpactGenerator),
        );
        registry.register(
            "traceability",
            Box::new(crate::inference::traceability::TraceabilityGenerator),
        );
        registry.register("drift", Box::new(crate::inference::drift::DriftGenerator));

        // Future: registry.register("ownership", Box::new(OwnershipGenerator));

        registry
    }

    /// Register a generator under a string key.
    pub fn register(&mut self, key: &str, generator: Box<dyn DeterministicInference>) {
        self.generators.insert(key.to_string(), generator);
    }

    /// Dispatch a query to the appropriate generator.
    pub fn dispatch(
        &self,
        evidence: &EngineeringEvidence,
        query_type: &QueryType,
    ) -> Option<EngineeringInsight> {
        let key = query_type_to_key(query_type);
        self.generators.get(&key).map(|gen| gen.generate(evidence))
    }
}

impl Default for InferenceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn query_type_to_key(qt: &QueryType) -> String {
    match qt {
        QueryType::WhyExists => "why_exists".to_string(),
        QueryType::Impact => "impact".to_string(),
        QueryType::Traceability => "traceability".to_string(),
        QueryType::Ownership => "ownership".to_string(),
        QueryType::Drift => "drift".to_string(),
        QueryType::Coverage => "coverage".to_string(),
        QueryType::Simulation => "simulation".to_string(),
    }
}
