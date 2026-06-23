use crate::models::ImpactReport;
use crate::traversal::dependency::DependencyTraverser;
use crate::traversal::TraversalConfig;
use ares_core::{AresError, NodeId, ProjectId};
use ares_store::repositories::graph::SqliteGraphRepository;
use std::sync::Arc;

pub struct ImpactAnalyzer {
    repo: Arc<SqliteGraphRepository>,
    traverser: DependencyTraverser,
}

impl ImpactAnalyzer {
    pub fn new(repo: Arc<SqliteGraphRepository>, config: TraversalConfig) -> Self {
        Self {
            repo: repo.clone(),
            traverser: DependencyTraverser::new(repo, config),
        }
    }

    /// Analyzes the impact of changing a specific node
    pub async fn analyze(
        &self,
        project_id: &ProjectId,
        node_id: &NodeId,
    ) -> Result<ImpactReport, AresError> {
        let trace = self.traverser.trace_dependents(project_id, node_id).await?;

        let mut affected_modules = Vec::new();
        let mut affected_functions = Vec::new();

        for node in trace.path {
            match node.node_type {
                ares_core::NodeType::File | ares_core::NodeType::Folder => {
                    affected_modules.push(node.label)
                }
                ares_core::NodeType::Function | ares_core::NodeType::Method => {
                    affected_functions.push(node.label)
                }
                _ => {}
            }
        }

        affected_modules.sort();
        affected_modules.dedup();
        affected_functions.sort();
        affected_functions.dedup();

        Ok(ImpactReport {
            target: node_id.as_str().to_string(),
            affected_modules,
            affected_functions,
            depth_analyzed: trace.depth,
        })
    }
}
