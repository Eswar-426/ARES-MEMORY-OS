use ares_reasoning::graph::ReasoningGraph;
use ares_reasoning::reports::generate_reports;
use ares_store::db::Store;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let store = Store::open(std::path::Path::new("ares_memory.db"))?;
    use ares_store::repositories::project::SqliteProjectRepository;
    let project_repo = SqliteProjectRepository::new(store.clone());
    let projects = project_repo.list_all()?;
    if projects.is_empty() {
        anyhow::bail!("No projects found in database.");
    }
    let project_id = projects[0].id.clone();

    let graph = ReasoningGraph::build(&store, &project_id)?;
    let output_dir = Path::new("artifacts/reasoning");
    
    generate_reports(&graph, output_dir)?;
    
    println!("Reasoning reports generated in artifacts/reasoning/");
    Ok(())
}
