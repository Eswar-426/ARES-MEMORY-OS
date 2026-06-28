pub mod assembly;
pub mod retrieval;
#[cfg(test)]
pub mod tests;
pub mod types;

use crate::assembly::PromptAssembler;
use crate::retrieval::ContextRetriever;
pub use crate::types::{ContextPackage, TokenBudget};
use ares_core::ProjectId;
use std::sync::Arc;
use tracing::info;

#[tracing::instrument(skip(retriever))]
pub async fn build_context<R: ContextRetriever + ?Sized>(
    query: &str,
    file_path: &str,
    retriever: Arc<R>,
    project_id: &ProjectId,
    budget: TokenBudget,
) -> anyhow::Result<ContextPackage> {
    let (decisions_res, git_res, ast_res, neighbors_res, own_res, arch_res, req_res) = tokio::join!(
        retriever.decisions(project_id, file_path),
        retriever.git_history(project_id, file_path),
        retriever.ast(project_id, file_path),
        retriever.neighbors(project_id, file_path),
        retriever.ownership(project_id, file_path),
        retriever.architecture(project_id, file_path),
        retriever.requirements(project_id, file_path),
    );

    // Provide empty defaults if retrieval fails to allow partial context
    let decisions =
        decisions_res.unwrap_or_else(|_| crate::types::DecisionContext { decisions: vec![] });
    let git = git_res.unwrap_or_else(|_| crate::types::GitContext { commits: vec![] });
    let ast = ast_res.unwrap_or_else(|_| crate::types::AstContext { nodes: vec![] });
    let neighbors =
        neighbors_res.unwrap_or_else(|_| crate::types::NeighborContext { nodes: vec![] });
    let ownership = own_res.unwrap_or_else(|_| crate::types::OwnershipContext { owners: vec![] });
    let architecture =
        arch_res.unwrap_or_else(|_| crate::types::ArchitectureContext { docs: vec![] });
    let requirements =
        req_res.unwrap_or_else(|_| crate::types::RequirementContext { reqs: vec![] });

    let dec_count = decisions.decisions.len();
    let git_count = git.commits.len();
    let ast_count = ast.nodes.len();
    let nbr_count = neighbors.nodes.len();

    let assembler = PromptAssembler::new(budget);
    let package = assembler.assemble(
        project_id.as_str(),
        file_path,
        query,
        ast,
        neighbors,
        git,
        ownership,
        architecture,
        requirements,
        decisions,
    );

    let trimmed_sections = if package.estimated_tokens < budget.as_usize() {
        "none".to_string()
    } else {
        "budget exceeded or heavily trimmed".to_string()
    };

    info!(
        "Context built\n\nQuery:\n\"{}\"\n\nFile:\n{}\n\nDecisions:\n{}\n\nCommits:\n{}\n\nFunctions:\n{}\n\nNeighbors:\n{}\n\nCharacters:\n{}\n\nEstimated Tokens:\n{}\n\nTrimmed:\n{}\n",
        query,
        file_path,
        dec_count,
        git_count,
        ast_count,
        nbr_count,
        package.assembled_prompt.chars().count(),
        package.estimated_tokens,
        trimmed_sections
    );

    Ok(package)
}

pub async fn build_context_with_store(
    query: &str,
    file_path: &str,
    store: &ares_store::Store,
    project_id: &ProjectId,
    budget: TokenBudget,
) -> anyhow::Result<ContextPackage> {
    let retriever = Arc::new(retrieval::StoreContextRetriever::new(store.clone()));
    build_context(query, file_path, retriever, project_id, budget).await
}
