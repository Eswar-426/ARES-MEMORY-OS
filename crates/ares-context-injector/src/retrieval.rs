#![allow(deprecated)]
use crate::types::{
    ArchitectureContext, AstContext, DecisionContext, GitCommit, GitContext, NeighborContext,
    OwnershipContext, RequirementContext,
};
use ares_core::{DecisionFilter, EdgeDirection, EdgeType, NodeType, ProjectId};
use ares_store::repositories::{
    decision::SqliteDecisionRepository, graph::SqliteGraphRepository,
    project::SqliteProjectRepository,
};
use ares_store::Store;
use async_trait::async_trait;
use std::collections::HashSet;
use std::path::PathBuf;

#[async_trait]
pub trait ContextRetriever: Send + Sync {
    async fn decisions(
        &self,
        project_id: &ProjectId,
        file_path: &str,
    ) -> anyhow::Result<DecisionContext>;
    async fn git_history(
        &self,
        project_id: &ProjectId,
        file_path: &str,
    ) -> anyhow::Result<GitContext>;
    async fn ast(&self, project_id: &ProjectId, file_path: &str) -> anyhow::Result<AstContext>;
    async fn neighbors(
        &self,
        project_id: &ProjectId,
        file_path: &str,
    ) -> anyhow::Result<NeighborContext>;
    async fn ownership(
        &self,
        project_id: &ProjectId,
        file_path: &str,
    ) -> anyhow::Result<OwnershipContext>;
    async fn architecture(
        &self,
        project_id: &ProjectId,
        file_path: &str,
    ) -> anyhow::Result<ArchitectureContext>;
    async fn requirements(
        &self,
        project_id: &ProjectId,
        file_path: &str,
    ) -> anyhow::Result<RequirementContext>;
}

pub struct StoreContextRetriever {
    store: Store,
}

impl StoreContextRetriever {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    fn get_project_path(&self, project_id: &ProjectId) -> anyhow::Result<PathBuf> {
        let repo = SqliteProjectRepository::new(self.store.clone());
        let project = repo
            .get_by_id(project_id)?
            .ok_or_else(|| anyhow::anyhow!("Project not found"))?;
        Ok(PathBuf::from(project.root_path))
    }
}

#[async_trait]
impl ContextRetriever for StoreContextRetriever {
    #[tracing::instrument(skip(self))]
    async fn decisions(
        &self,
        project_id: &ProjectId,
        file_path: &str,
    ) -> anyhow::Result<DecisionContext> {
        let repo = SqliteDecisionRepository::new(self.store.clone());
        let filter = DecisionFilter {
            file_path: Some(file_path.to_string()),
            ..Default::default()
        };

        let decisions = repo.list(project_id, filter)?;
        Ok(DecisionContext { decisions })
    }

    #[tracing::instrument(skip(self))]
    async fn git_history(
        &self,
        project_id: &ProjectId,
        file_path: &str,
    ) -> anyhow::Result<GitContext> {
        let project_path = self.get_project_path(project_id)?;
        let captured_at = ares_core::types::event::now_micros();

        let (nodes, edges) = tokio::task::block_in_place(|| {
            ares_git_memory::commits::CommitExtractor::extract(
                &project_path,
                project_id,
                500,
                captured_at,
            )
        })
        .map_err(|e| anyhow::anyhow!("Git extraction failed: {}", e))?;

        let file_node_id = ares_core::canonicalize_node_id(file_path);

        let touching_commits: HashSet<String> = edges
            .into_iter()
            .filter(|e| e.edge_type == EdgeType::Touches && e.to_node_id.as_str() == file_node_id)
            .map(|e| e.from_node_id.as_str().to_string())
            .collect();

        let mut commits = Vec::new();
        for node in nodes {
            if node.node_type == NodeType::Commit && touching_commits.contains(node.id.as_str()) {
                let props = &node.properties;
                let hash = props
                    .get("hash")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let author = props
                    .get("author")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let message = props
                    .get("subject")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                commits.push(GitCommit {
                    hash,
                    author,
                    message,
                    timestamp: node.created_at,
                });
            }
        }

        commits.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
        commits.truncate(5);

        Ok(GitContext { commits })
    }

    #[tracing::instrument(skip(self))]
    async fn ast(&self, project_id: &ProjectId, file_path: &str) -> anyhow::Result<AstContext> {
        let repo = SqliteGraphRepository::new(self.store.clone());
        let nodes = repo.get_by_file_path(project_id, file_path)?;

        let mut ast_nodes = Vec::new();
        for node in nodes {
            match node.node_type {
                NodeType::Function
                | NodeType::Class
                | NodeType::Trait
                | NodeType::Struct
                | NodeType::Module => {
                    ast_nodes.push(node);
                }
                _ => {}
            }
        }
        Ok(AstContext { nodes: ast_nodes })
    }

    #[tracing::instrument(skip(self))]
    async fn neighbors(
        &self,
        project_id: &ProjectId,
        file_path: &str,
    ) -> anyhow::Result<NeighborContext> {
        let repo = SqliteGraphRepository::new(self.store.clone());
        let file_node_id = ares_core::NodeId::from(ares_core::canonicalize_node_id(file_path));

        let edge_types = vec![
            EdgeType::Imports,
            EdgeType::Defines,
            EdgeType::Calls,
            EdgeType::Extends,
            EdgeType::DependsOn,
            EdgeType::Implements,
            EdgeType::Uses,
            EdgeType::Contains,
            EdgeType::ContainedIn,
            EdgeType::References,
            EdgeType::Touches,
        ];

        let mut nodes = repo
            .get_neighbors(&file_node_id, EdgeDirection::Both, &edge_types)
            .unwrap_or_default();
        // Remove the file itself from neighbors
        nodes.retain(|n| n.id != file_node_id);
        Ok(NeighborContext { nodes })
    }

    #[tracing::instrument(skip(self))]
    async fn ownership(
        &self,
        project_id: &ProjectId,
        file_path: &str,
    ) -> anyhow::Result<OwnershipContext> {
        let repo = SqliteGraphRepository::new(self.store.clone());
        let file_node_id = ares_core::NodeId::from(ares_core::canonicalize_node_id(file_path));

        let edge_types = vec![EdgeType::OwnedBy, EdgeType::AuthoredBy, EdgeType::Maintains];
        let nodes = repo
            .get_neighbors(&file_node_id, EdgeDirection::Outgoing, &edge_types)
            .unwrap_or_default();
        
        Ok(OwnershipContext { owners: nodes })
    }

    #[tracing::instrument(skip(self))]
    async fn architecture(
        &self,
        project_id: &ProjectId,
        file_path: &str,
    ) -> anyhow::Result<ArchitectureContext> {
        let repo = SqliteGraphRepository::new(self.store.clone());
        let file_node_id = ares_core::NodeId::from(ares_core::canonicalize_node_id(file_path));

        let edge_types = vec![EdgeType::References, EdgeType::Contains, EdgeType::ContainedIn];
        let nodes = repo
            .get_neighbors(&file_node_id, EdgeDirection::Both, &edge_types)
            .unwrap_or_default()
            .into_iter()
            .filter(|n| n.node_type == NodeType::Architecture || n.node_type == NodeType::Concept)
            .collect();
        
        Ok(ArchitectureContext { docs: nodes })
    }

    #[tracing::instrument(skip(self))]
    async fn requirements(
        &self,
        project_id: &ProjectId,
        file_path: &str,
    ) -> anyhow::Result<RequirementContext> {
        let repo = SqliteGraphRepository::new(self.store.clone());
        let file_node_id = ares_core::NodeId::from(ares_core::canonicalize_node_id(file_path));

        let edge_types = vec![EdgeType::Satisfies, EdgeType::References, EdgeType::MotivatedBy];
        let nodes = repo
            .get_neighbors(&file_node_id, EdgeDirection::Outgoing, &edge_types)
            .unwrap_or_default()
            .into_iter()
            .filter(|n| n.node_type == NodeType::Requirement || n.node_type == NodeType::Feature)
            .collect();
        
        Ok(RequirementContext { reqs: nodes })
    }
}
