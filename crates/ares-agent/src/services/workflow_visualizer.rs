use ares_core::types::workflow_api::WorkflowGraphResponse;
use ares_core::{AresError, WorkflowDefinition};
use ares_store::repositories::traits::WorkflowRepository;
use std::sync::Arc;

pub struct WorkflowVisualizer {
    repo: Arc<dyn WorkflowRepository + Send + Sync>,
}

impl WorkflowVisualizer {
    pub fn new(repo: Arc<dyn WorkflowRepository + Send + Sync>) -> Self {
        Self { repo }
    }

    pub fn visualize(&self, version_id: &str) -> Result<WorkflowGraphResponse, AresError> {
        // 1. Check cache first
        if let Some(cached_json) = self.repo.get_visualization(version_id)? {
            if let Ok(cached_resp) = serde_json::from_str::<WorkflowGraphResponse>(&cached_json) {
                return Ok(cached_resp);
            }
        }

        // 2. Generate if not cached
        let def_json = self.repo.get_version_definition(version_id)?;
        let def: WorkflowDefinition = serde_json::from_str(&def_json)?;

        // Size Guard
        let node_count = def.steps.len();
        let visualization_truncated = node_count > 5000;

        let mermaid = if visualization_truncated {
            None
        } else {
            Some(self.generate_mermaid(&def))
        };

        let graph_json = self.generate_graph_json(&def);

        let resp = WorkflowGraphResponse {
            workflow_version_id: version_id.to_string(),
            mermaid,
            graph_json,
            visualization_truncated,
        };

        let full_json = serde_json::to_string(&resp).unwrap_or_default();

        // 3. Save to cache
        self.repo.save_visualization(version_id, &full_json)?;

        Ok(resp)
    }

    fn generate_mermaid(&self, def: &WorkflowDefinition) -> String {
        let mut mermaid = String::from("graph TD\n");
        for step in &def.steps {
            for dep in &step.depends_on {
                mermaid.push_str(&format!("    {} --> {}\n", dep.as_str(), step.id.as_str()));
            }
        }
        mermaid
    }

    fn generate_graph_json(&self, def: &WorkflowDefinition) -> serde_json::Value {
        let nodes: Vec<_> = def.steps.iter().map(|s| s.id.as_str()).collect();
        let mut edges = Vec::new();
        for step in &def.steps {
            for dep in &step.depends_on {
                edges.push(
                    serde_json::json!({ "source": dep.as_str(), "target": step.id.as_str() }),
                );
            }
        }
        serde_json::json!({
            "nodes": nodes,
            "edges": edges
        })
    }
}
