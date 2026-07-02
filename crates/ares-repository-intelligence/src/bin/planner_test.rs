use ares_repository_intelligence::core::capabilities::Capability;
use ares_repository_intelligence::core::context::{
    CachePolicy, EntryPoint, ExecutionContext, ExecutionMode, ExecutionPolicy, RepositoryContext,
    RepositoryInfo, RepositorySnapshot, RequestContext, WorkspaceContext,
};
use ares_repository_intelligence::core::engine::EngineId;
use ares_repository_intelligence::engines::graph::RepositoryGraphEngine;
use ares_repository_intelligence::engines::overview::RepositoryOverviewEngine;
use ares_repository_intelligence::planner::pipeline::ExecutionPlanner;
use ares_repository_intelligence::planner::registry::EngineRegistry;
use ares_store::Store;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("debug,ares_repository_intelligence=debug")
        .init();

    tracing::info!("Initializing Planner Test Harness...");

    // 2. Setup mock Store and Context
    let store = Store::open(&PathBuf::from(":memory:")).unwrap();
    let repository = RepositoryInfo {
        root_path: "mock_path".to_string(),
        name: "Mock Project".to_string(),
    };

    let context = RepositoryContext {
        repository,
        snapshot: RepositorySnapshot::default(),
        workspace: WorkspaceContext {
            workspace_id: uuid::Uuid::new_v4().to_string(),
        },
        execution: ExecutionContext {
            execution_id: uuid::Uuid::new_v4().to_string(),
            started_at: 0,
            requested_by: "planner_test".to_string(),
            entry_point: EntryPoint::CLI,
            execution_mode: ExecutionMode::Direct,
            streaming: false,
            debug: true,
        },
        policy: ExecutionPolicy {
            cache_policy: CachePolicy::BypassCache,
            timeout_ms: 30000,
            ..ExecutionPolicy::default()
        },
        request: RequestContext {
            query: "Analyze this repository".to_string(),
            parameters: std::collections::HashMap::new(),
        },
    };

    // 3. Register engines
    let mut registry = EngineRegistry::new();

    registry.register(
        EngineId::Overview,
        vec![Capability::Workspace],
        Box::new(RepositoryOverviewEngine::new(store.clone())),
    );

    registry.register(
        EngineId::Graph,
        vec![Capability::GraphSearch, Capability::Traceability],
        Box::new(RepositoryGraphEngine::new(store.clone())),
    );

    // 4. Run ExecutionPlanner
    let planner = ExecutionPlanner::new(&registry);

    tracing::info!("Executing Planner pipeline...");
    let response = planner.execute(&context).await;

    tracing::info!("Planner execution completed successfully!");
    tracing::info!("Schema version: {}", response.schema_version);
    tracing::info!("Planner version: {}", response.planner_version);
    tracing::info!(
        "Final Evidence Bundle nodes: {}",
        response
            .evidence
            .graph
            .as_ref()
            .map(|g| g.nodes.len())
            .unwrap_or(0)
    );
    tracing::info!(
        "Planner trace events: {}",
        response.planner_trace.events.len()
    );
    tracing::info!("Final Artifacts: {:?}", response.artifacts);

    Ok(())
}
