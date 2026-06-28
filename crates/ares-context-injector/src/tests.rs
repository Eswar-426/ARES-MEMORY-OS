#![allow(deprecated)]
use crate::assembly::PromptAssembler;
use crate::build_context;
use crate::retrieval::ContextRetriever;
use crate::types::{
    ArchitectureContext, AstContext, DecisionContext, GitCommit, GitContext, NeighborContext,
    OwnershipContext, RequirementContext, TokenBudget,
};
use ares_core::{Decision, DecisionId, GraphNode, NodeId, NodeType, ProjectId};
use async_trait::async_trait;
use std::sync::Arc;

struct MockRetriever {
    decisions_ret: DecisionContext,
    git_ret: GitContext,
    ast_ret: AstContext,
    neighbors_ret: NeighborContext,
    ownership_ret: OwnershipContext,
    architecture_ret: ArchitectureContext,
    requirements_ret: RequirementContext,
}

#[async_trait]
impl ContextRetriever for MockRetriever {
    async fn decisions(
        &self,
        _project_id: &ProjectId,
        _file_path: &str,
    ) -> anyhow::Result<DecisionContext> {
        Ok(self.decisions_ret.clone())
    }
    async fn git_history(
        &self,
        _project_id: &ProjectId,
        _file_path: &str,
    ) -> anyhow::Result<GitContext> {
        Ok(self.git_ret.clone())
    }
    async fn ast(&self, _project_id: &ProjectId, _file_path: &str) -> anyhow::Result<AstContext> {
        Ok(self.ast_ret.clone())
    }
    async fn neighbors(
        &self,
        _project_id: &ProjectId,
        _file_path: &str,
    ) -> anyhow::Result<NeighborContext> {
        Ok(self.neighbors_ret.clone())
    }
    async fn ownership(
        &self,
        _project_id: &ProjectId,
        _file_path: &str,
    ) -> anyhow::Result<OwnershipContext> {
        Ok(self.ownership_ret.clone())
    }
    async fn architecture(
        &self,
        _project_id: &ProjectId,
        _file_path: &str,
    ) -> anyhow::Result<ArchitectureContext> {
        Ok(self.architecture_ret.clone())
    }
    async fn requirements(
        &self,
        _project_id: &ProjectId,
        _file_path: &str,
    ) -> anyhow::Result<RequirementContext> {
        Ok(self.requirements_ret.clone())
    }
}

fn make_node(id: &str, node_type: NodeType, label: &str) -> GraphNode {
    GraphNode {
        id: NodeId::from(id.to_string()),
        project_id: ProjectId::new(),
        node_type,
        label: label.to_string(),
        properties: serde_json::json!({}),
        file_path: None,
        created_at: 0,
        updated_at: 0,
        deleted_at: None,
    }
}

fn default_mock_retriever() -> MockRetriever {
    MockRetriever {
        decisions_ret: DecisionContext {
            decisions: vec![Decision {
                id: DecisionId::from("ADR-001".to_string()),
                project_id: ProjectId::new(),
                memory_id: ares_core::MemoryId::new(),
                title: "Use Postgres".to_string(),
                decision_text: "We will use PG".to_string(),
                reason: "".to_string(),
                status: ares_core::DecisionStatus::Accepted,
                confidence: 1.0,
                reasoning_steps: vec![],
                alternatives: vec![],
                risks: vec![],
                context_snapshot: ares_core::types::decision::ContextSnapshot::default(),
                future_impact: ares_core::types::decision::FutureImpact::default(),
                files_impacted: vec![],
                services_impacted: vec![],
                supersedes: vec![],
                superseded_by: None,
                decided_by: "Alice".to_string(),
                discussed_in: vec![],
                review_due_at: None,
                last_reviewed_at: None,
                created_at: 100,
                updated_at: 100,
            }],
        },
        git_ret: GitContext {
            commits: vec![GitCommit {
                hash: "abcdef".to_string(),
                author: "Bob".to_string(),
                message: "Initial commit".to_string(),
                timestamp: 200,
            }],
        },
        ast_ret: AstContext {
            nodes: vec![make_node("fn1", NodeType::Function, "calculate")],
        },
        neighbors_ret: NeighborContext {
            nodes: vec![make_node("file2", NodeType::File, "utils.rs")],
        },
        ownership_ret: OwnershipContext { owners: vec![] },
        architecture_ret: ArchitectureContext { docs: vec![] },
        requirements_ret: RequirementContext { reqs: vec![] },
    }
}

#[tokio::test]
async fn test_determinism() {
    let retriever = Arc::new(default_mock_retriever());
    let project_id = ProjectId::new();

    let mut res1 = build_context(
        "query",
        "file.rs",
        retriever.clone(),
        &project_id,
        TokenBudget::Maximum,
    )
    .await
    .unwrap();
    let mut res2 = build_context(
        "query",
        "file.rs",
        retriever.clone(),
        &project_id,
        TokenBudget::Maximum,
    )
    .await
    .unwrap();

    // Normalize dynamic timestamp
    let generated_line1 = res1
        .assembled_prompt
        .lines()
        .find(|l| l.starts_with("Generated:"))
        .unwrap()
        .to_string();
    let generated_line2 = res2
        .assembled_prompt
        .lines()
        .find(|l| l.starts_with("Generated:"))
        .unwrap()
        .to_string();
    res1.assembled_prompt = res1
        .assembled_prompt
        .replace(&generated_line1, "Generated: TIME");
    res2.assembled_prompt = res2
        .assembled_prompt
        .replace(&generated_line2, "Generated: TIME");

    assert_eq!(res1.assembled_prompt, res2.assembled_prompt);
    assert_eq!(res1.sources, res2.sources);
    assert!((res1.estimated_tokens as i64 - res2.estimated_tokens as i64).abs() <= 5);
}

#[tokio::test]
async fn test_empty_repository() {
    let empty_retriever = Arc::new(MockRetriever {
        decisions_ret: DecisionContext { decisions: vec![] },
        git_ret: GitContext { commits: vec![] },
        ast_ret: AstContext { nodes: vec![] },
        neighbors_ret: NeighborContext { nodes: vec![] },
        ownership_ret: OwnershipContext { owners: vec![] },
        architecture_ret: ArchitectureContext { docs: vec![] },
        requirements_ret: RequirementContext { reqs: vec![] },
    });

    let project_id = ProjectId::new();
    let res = build_context(
        "query",
        "file.rs",
        empty_retriever,
        &project_id,
        TokenBudget::Maximum,
    )
    .await
    .unwrap();

    assert!(res.assembled_prompt.contains("None found."));
    assert!(res.assembled_prompt.contains("ENGINEERING DECISIONS"));
    assert!(res.sources.is_empty());
}

#[tokio::test]
async fn test_large_repository_trimming() {
    let mut large_retriever = default_mock_retriever();
    // Add 100 neighbors
    for i in 0..100 {
        large_retriever.neighbors_ret.nodes.push(make_node(
            &format!("n{}", i),
            NodeType::Class,
            &format!("Class{}", i),
        ));
    }

    let _retriever = Arc::new(large_retriever);
    let project_id = ProjectId::new();

    let assembler = PromptAssembler::new(TokenBudget::Small);

    let mut ast_nodes = Vec::new();
    for i in 0..1000 {
        // 1000 nodes = 5000 tokens
        ast_nodes.push(make_node(
            &format!("ast{}", i),
            NodeType::Function,
            &format!("long_function_name_that_takes_up_tokens_{}", i),
        ));
    }
    let ast_context = AstContext { nodes: ast_nodes };

    let package = assembler.assemble(
        project_id.as_str(),
        "file.rs",
        "query",
        ast_context,
        NeighborContext { nodes: vec![] },
        GitContext { commits: vec![] },
        OwnershipContext { owners: vec![] },
        ArchitectureContext { docs: vec![] },
        RequirementContext { reqs: vec![] },
        DecisionContext { decisions: vec![] },
    );

    assert!(package.estimated_tokens <= TokenBudget::Small.as_usize());
}

#[tokio::test]
async fn test_duplicate_sources() {
    let mut dup_retriever = default_mock_retriever();
    dup_retriever
        .decisions_ret
        .decisions
        .push(dup_retriever.decisions_ret.decisions[0].clone()); // duplicate decision

    let retriever = Arc::new(dup_retriever);
    let project_id = ProjectId::new();

    let res = build_context(
        "query",
        "file.rs",
        retriever,
        &project_id,
        TokenBudget::Maximum,
    )
    .await
    .unwrap();

    let adr_count = res
        .sources
        .iter()
        .filter(|s| s.as_str() == "decision:ADR-001")
        .count();
    assert_eq!(adr_count, 1);
}

#[tokio::test]
async fn test_unicode() {
    let mut uni_retriever = default_mock_retriever();
    uni_retriever
        .ast_ret
        .nodes
        .push(make_node("u1", NodeType::Function, "Δcalculate"));
    uni_retriever
        .ast_ret
        .nodes
        .push(make_node("u2", NodeType::Function, "用户登录"));

    let retriever = Arc::new(uni_retriever);
    let project_id = ProjectId::new();

    let res = build_context(
        "query",
        "file.rs",
        retriever,
        &project_id,
        TokenBudget::Maximum,
    )
    .await
    .unwrap();

    assert!(res.assembled_prompt.contains("Δcalculate"));
    assert!(res.assembled_prompt.contains("用户登录"));
    // Tokens estimation for unicode
    assert!(res.estimated_tokens > 0);
}
