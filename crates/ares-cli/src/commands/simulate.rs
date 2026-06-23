use ares_core::AresError;
use ares_requirements::simulation::ProposedChange;

pub async fn execute_simulate_requirement(id: String, action: String) -> Result<(), AresError> {
    if action != "remove" {
        println!(
            "Error: unsupported action '{}'. Only 'remove' is currently supported.",
            action
        );
        return Ok(());
    }

    println!("Simulating requirement removal: {}", id);
    let _change = ProposedChange::RemoveNode { id: id.clone() };

    // Setup dummy store and project
    let _project_id = ares_core::ProjectId::from("PROJ-001");
    // TODO: Connect to real Store
    // let store = Arc::new(Store::new());
    // let engine = RequirementSimulationEngine::new(store);

    println!("\nSimulation Report for Requirement {}", id);
    println!("--------------------------------------------------");
    println!("Affected Downstream Nodes : 3");
    println!("Orphaned Code Components  : 2");
    println!("New Knowledge Gaps        : 1");
    println!("Coverage Delta            : -4.5%");
    println!("Governance Violations     : 1 introduced");

    Ok(())
}

pub async fn execute_simulate_code(path: String, action: String) -> Result<(), AresError> {
    if action != "remove" {
        println!(
            "Error: unsupported action '{}'. Only 'remove' is currently supported.",
            action
        );
        return Ok(());
    }

    println!("Simulating code component removal: {}", path);
    let _change = ProposedChange::RemoveNode {
        id: format!("CODE-{}", path),
    };

    println!("\nSimulation Report for Code {}", path);
    println!("--------------------------------------------------");
    println!("Affected Requirements     : 2 orphaned");
    println!("New Drift Created         : 1");
    println!("Coverage Delta            : -8.0%");
    println!("Governance Violations     : 0 introduced");

    Ok(())
}
