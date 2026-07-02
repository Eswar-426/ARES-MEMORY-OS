use crate::core::capabilities::Capability;
use crate::core::context::RepositoryContext;
use crate::core::engine::{
    EngineDescriptor, EngineExecutionResult, EngineId, EngineInput, RepositoryEngine,
};
use crate::core::errors::{EngineError, EngineResult};
use crate::core::evidence::EvidenceBundle;
use crate::core::metadata::ExecutionMetadata;
use ares_core::ProjectId;
use ares_git_memory::GitMemoryExtractor;
use async_trait::async_trait;
use std::collections::HashMap;

pub struct RepositoryGitEngine;

impl Default for RepositoryGitEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RepositoryGitEngine {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RepositoryEngine for RepositoryGitEngine {
    fn descriptor(&self) -> EngineDescriptor {
        EngineDescriptor {
            id: EngineId::GitMemory,
            version: "0.1.0".to_string(),
            capabilities: vec![Capability::GitHistory],
            planner_api_version: 1,
        }
    }

    async fn execute(
        &self,
        context: &RepositoryContext,
        _input: EngineInput,
    ) -> EngineResult<EngineExecutionResult> {
        let start = std::time::Instant::now();
        let project_path = std::path::PathBuf::from(&context.repository.root_path);

        let extractor = GitMemoryExtractor::new(&project_path);
        let project_id = ProjectId::from("mock_id");

        let git_result = extractor.extract(&project_id).map_err(|e| {
            EngineError::ExecutionError(format!("GitMemory extraction failed: {}", e))
        })?;

        let mut bundle = EvidenceBundle::default();

        if let Some(graph) = &mut bundle.graph {
            graph
                .nodes
                .extend(git_result.nodes.iter().map(|n| n.id.to_string()));
            graph
                .edges
                .extend(git_result.edges.iter().map(|e| e.id.to_string()));
        } else {
            bundle.graph = Some(crate::core::evidence::GraphEvidence {
                nodes: git_result.nodes.iter().map(|n| n.id.to_string()).collect(),
                edges: git_result.edges.iter().map(|e| e.id.to_string()).collect(),
                paths: vec![],
            });
        }

        let mut diagnostics = HashMap::new();
        for source in git_result.sources {
            diagnostics.insert(
                source.name.clone(),
                format!(
                    "Available: {}, Captured: {}, Nodes: {}, Edges: {}",
                    source.available, source.captured, source.node_count, source.edge_count
                ),
            );
        }

        Ok(EngineExecutionResult {
            descriptor: self.descriptor(),
            engine_id: EngineId::GitMemory,
            capability: Capability::GitHistory,
            evidence: bundle,
            metadata: ExecutionMetadata {
                engine: "GitMemory".to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
                cache_hit: false,
                confidence: 1.0,
                errors: vec![],
                warnings: vec![],
                retry_count: 0,
                sources_used: vec!["git".to_string()],
            },
            diagnostics,
            warnings: vec![],
            errors: vec![],
            artifacts: vec![],
        })
    }
}
