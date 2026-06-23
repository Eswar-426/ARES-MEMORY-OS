use crate::models::RepositoryConfidence;
use ares_core::NodeId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryPurpose {
    pub purpose: String,
    pub confidence: f32,
    pub source_nodes: Vec<NodeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub requirement_nodes: Vec<NodeId>,
    pub decision_nodes: Vec<NodeId>,
    pub architecture_nodes: Vec<NodeId>,
    pub code_nodes: Vec<NodeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMap {
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureLayer {
    pub name: String,
    pub nodes: Vec<NodeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureTopology {
    pub layers: Vec<ArchitectureLayer>,
    pub critical_components: Vec<NodeId>,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceBoundary {
    pub name: String,
    pub isolated_nodes: Vec<NodeId>,
    pub interface_nodes: Vec<NodeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceBoundaries {
    pub boundaries: Vec<ServiceBoundary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipArea {
    pub owner_id: NodeId,
    pub capability_names: Vec<String>,
    pub code_nodes: Vec<NodeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnershipMap {
    pub areas: Vec<OwnershipArea>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyMap {
    pub external_dependencies: Vec<String>,
    pub internal_dependencies: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSources {
    pub top_nodes: Vec<NodeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryState {
    pub purpose: RepositoryPurpose,
    pub capabilities: CapabilityMap,
    pub architecture: ArchitectureTopology,
    pub boundaries: ServiceBoundaries,
    pub ownership: OwnershipMap,
    pub dependencies: DependencyMap,
    pub confidence: RepositoryConfidence,
    pub evidence: EvidenceSources,
}
