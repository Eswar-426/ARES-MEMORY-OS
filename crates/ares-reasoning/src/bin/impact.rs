use ares_core::{NodeId, ProjectId};
use ares_reasoning::graph::ReasoningGraph;
use ares_reasoning::impact::ImpactAnalyzer;
use ares_store::db::Store;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    node: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let node_id = NodeId::from(args.node);
    
    let store = Store::open(std::path::Path::new("ares_memory.db"))?;
    // We assume there's one project or we pass project_id. For now just grab the first project.
    use ares_store::repositories::project::SqliteProjectRepository;
    let project_repo = SqliteProjectRepository::new(store.clone());
    let projects = project_repo.list_all()?;
    if projects.is_empty() {
        anyhow::bail!("No projects found in database.");
    }
    let project_id = projects[0].id.clone();
    
    let graph = ReasoningGraph::build(&store, &project_id)?;
    let report = ImpactAnalyzer::analyze(&graph, &node_id);
    
    let json = serde_json::to_string_pretty(&report)?;
    println!("{}", json);
    
    Ok(())
}
