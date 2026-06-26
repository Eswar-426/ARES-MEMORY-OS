use serde::{Deserialize, Serialize};

/// A descriptor for an ARES intelligence capability.
/// Used by the `/capabilities` endpoint for enterprise introspection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDescriptor {
    pub id: String,
    pub name: String,
    pub owner_crate: String,
    pub version: String,
    pub phase: String,
}

/// The static registry of all known ARES intelligence capabilities.
/// This must be updated as new phases are introduced.
pub fn registered_capabilities() -> Vec<CapabilityDescriptor> {
    vec![
        CapabilityDescriptor {
            id: "traceability".into(),
            name: "Requirement Traceability".into(),
            owner_crate: "ares-traceability".into(),
            version: "0.1.0".into(),
            phase: "P2".into(),
        },
        CapabilityDescriptor {
            id: "decision_intelligence".into(),
            name: "Decision Intelligence".into(),
            owner_crate: "ares-decision-intelligence".into(),
            version: "0.1.0".into(),
            phase: "P8".into(),
        },
        CapabilityDescriptor {
            id: "repository_intelligence".into(),
            name: "Repository Intelligence".into(),
            owner_crate: "ares-repository-intelligence".into(),
            version: "0.1.0".into(),
            phase: "P9".into(),
        },
        CapabilityDescriptor {
            id: "query_layer".into(),
            name: "Memory Query Layer".into(),
            owner_crate: "ares-query".into(),
            version: "0.1.0".into(),
            phase: "P10".into(),
        },
        CapabilityDescriptor {
            id: "bootstrap_intelligence".into(),
            name: "Bootstrap Intelligence".into(),
            owner_crate: "ares-memory-bootstrap".into(),
            version: "0.1.0".into(),
            phase: "P12".into(),
        },
        CapabilityDescriptor {
            id: "lifecycle_intelligence".into(),
            name: "Memory Lifecycle Intelligence".into(),
            owner_crate: "ares-memory-lifecycle".into(),
            version: "0.1.0".into(),
            phase: "P13".into(),
        },
    ]
}
