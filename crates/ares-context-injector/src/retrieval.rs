use crate::types::{ContextPackage, TokenBudget};
use ares_core::types::pagination::Pagination;
use ares_core::{AresError, DecisionFilter, NodeType, ProjectId};
use ares_store::{
    repositories::{
        decision::SqliteDecisionRepository, graph::SqliteGraphRepository,
        project::SqliteProjectRepository,
    },
    Store,
};
use tracing::info;

pub struct ContextSelector {
    store: Store,
}

impl ContextSelector {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub async fn build_package(
        &self,
        project_id: &str,
        prompt: &str,
        _budget: TokenBudget,
    ) -> Result<ContextPackage, AresError> {
        info!("Building context package for project_id={}", project_id);

        let p_id = ProjectId::from(project_id.to_string());

        // 1. Snapshot retrieval
        let project_repo = SqliteProjectRepository::new(self.store.clone());
        let _project = project_repo.get_by_id(&p_id)?;

        // 2. Architecture retrieval
        let graph_repo = SqliteGraphRepository::new(self.store.clone());

        let pagination = Pagination {
            page: 1,
            page_size: 50,
        };

        // We will fetch crates and modules to simulate architecture retrieval
        let arch_page =
            graph_repo.list_nodes_paginated(&p_id, Some(NodeType::Module), None, &pagination)?;
        let mut architecture_nodes = arch_page.items;

        if let Ok(crate_page) =
            graph_repo.list_nodes_paginated(&p_id, Some(NodeType::Service), None, &pagination)
        {
            architecture_nodes.extend(crate_page.items);
        }

        // 3. Decision retrieval
        let decision_repo = SqliteDecisionRepository::new(self.store.clone());
        let decision_filter = DecisionFilter::default();
        let decisions = decision_repo.list(&p_id, decision_filter)?;

        // 4. Bug history retrieval
        let bug_page =
            graph_repo.list_nodes_paginated(&p_id, Some(NodeType::Bug), None, &pagination)?;
        let bugs = bug_page.items;

        // 5. Semantic / Memory retrieval (Generic context matching prompt)
        let memory_page =
            graph_repo.list_nodes_paginated(&p_id, None, Some(prompt), &pagination)?;
        let memories = memory_page.items;

        Ok(ContextPackage {
            project_id: project_id.to_string(),
            original_prompt: prompt.to_string(),
            architecture_nodes,
            decisions,
            bugs,
            memories,
            assembled_prompt: String::new(),
            estimated_tokens: 0,
        })
    }
}
