use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::capabilities::Capability;
use crate::core::context::RepositoryContext;
use crate::core::errors::EngineResult;
use crate::core::evidence::RawEvidenceBundle;
use crate::core::metadata::ExecutionMetadata;

/// Stable identity for engine implementations.
/// Multiple engines may satisfy the same capability — the planner shouldn't
/// care which implementation was selected.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EngineId {
    Overview,
    Graph,
    GitMemory,
    WhyExists,
    Impact,
    Traceability,
    Simulation,
    Conversation,
    Workspace,
    SelfTest,
}

/// Versioned engine descriptor. Enables planner replay to know exactly
/// which engine version produced a given piece of evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineDescriptor {
    pub id: EngineId,
    pub version: String,
    pub capabilities: Vec<Capability>,
    pub planner_api_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub format: String,
    pub content: String,
}

/// Raw result from a single engine execution.
/// Evidence is always `RawEvidenceBundle` — never processed at this stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineExecutionResult {
    pub engine_id: EngineId,
    pub descriptor: EngineDescriptor,
    pub capability: Capability,
    pub evidence: RawEvidenceBundle,
    pub metadata: ExecutionMetadata,
    pub diagnostics: HashMap<String, String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineInput {
    NodeId(String),
    Query(String),
    None,
}

/// The contract every engine implements. Frozen API.
#[async_trait::async_trait]
pub trait RepositoryEngine: Send + Sync {
    fn descriptor(&self) -> EngineDescriptor;

    async fn execute(
        &self,
        context: &RepositoryContext,
        input: EngineInput,
    ) -> EngineResult<EngineExecutionResult>;
}
