use ares_agent_arena::agents::baseline::BaselineAgent;
use ares_agent_arena::agents::context_aware::ContextAwareAgent;
use ares_agent_arena::agents::enhanced::EnhancedContextAgent;
use ares_agent_arena::agents::planner::PlannerAgentStub;
use ares_agent_arena::executor::ArenaExecutor;
use ares_agent_arena::models::ArenaTask;
use ares_agent_arena::report::ReportGenerator;
use ares_context::context_engine::ContextEngine;
use ares_context::pack::ContextPackBuilder;
use std::fs;
use std::sync::Arc;
use ares_core::ProjectId;
use ares_store::db::Store;
use ares_store::repositories::graph::SqliteGraphRepository;
use ares_store::repositories::memory::SqliteMemoryRepository;
use ares_store::repositories::project::SqliteProjectRepository;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting ARES Agent Arena Benchmark...");

    let workspace_root = std::env::current_dir()?;
    let store = Store::open(&workspace_root.join("ares_memory.db"))?;

    let graph_repo = Arc::new(SqliteGraphRepository::new(store.clone()));
    let memory_repo = Arc::new(SqliteMemoryRepository::new(store.clone()));
    let project_repo = SqliteProjectRepository::new(store.clone());

    // We'll just grab the first project for benchmarking
    let mut projects = project_repo.list_all()?;
    let project_id = if projects.is_empty() {
        println!("No projects found. Creating a dummy project for benchmarking.");
        let p_id = ares_core::ProjectId::new();
        let proj = ares_core::types::project::Project {
            id: p_id.clone(),
            name: "ARES_Mock".to_string(),
            description: "Mock project".to_string(),
            root_path: workspace_root.to_string_lossy().to_string(),
            primary_language: "rust".to_string(),
            domain: "benchmark".to_string(),
            maturity: ares_core::types::project::ProjectMaturity::Greenfield,
            created_at: 0,
            updated_at: 0,
            deleted_at: None,
        };
        project_repo.create(&proj)?;
        p_id
    } else {
        projects.remove(0).id
    };

    let engine = Arc::new(ContextEngine::new(
        project_id.clone(),
        graph_repo.clone(),
        memory_repo.clone(),
        ares_context::traversal::TraversalConfig::default(),
    ));

    let budget = ares_context::models::pack::ContextBudget::default();
    let builder = Arc::new(ContextPackBuilder::new(budget));

    let executor = ArenaExecutor {
        baseline: BaselineAgent {
            graph_repo: graph_repo.clone(),
            project_id: project_id.clone(),
        },
        context_aware: ContextAwareAgent {
            engine: engine.clone(),
            builder: builder.clone(),
        },
        enhanced: EnhancedContextAgent {
            engine: engine.clone(),
            builder: builder.clone(),
            graph_repo: graph_repo.clone(),
            project_id: project_id.clone(),
        },
        planner: PlannerAgentStub {},
    };

    // Load dataset
    let dataset_path = workspace_root.join("datasets").join("agent_tasks.json");
    let content = fs::read_to_string(&dataset_path)?;
    let tasks: Vec<ArenaTask> = serde_json::from_str(&content)?;

    println!("Loaded {} tasks from dataset.", tasks.len());

    let mut reports = Vec::new();

    for task in tasks {
        println!("Executing Task: {}", task.title);
        let report = executor.execute_task(&task).await?;
        reports.push(report);
    }

    let report_gen = ReportGenerator::new(workspace_root.to_str().unwrap());
    report_gen.save_json(&reports)?;
    report_gen.generate_markdown(&reports)?;

    println!("Benchmark complete! Reports saved to artifacts/arena/.");
    Ok(())
}
