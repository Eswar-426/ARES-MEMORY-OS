use std::sync::Arc;
use ares_core::AresError;
use ares_store::db::Store;

pub async fn execute_exemptions() -> Result<(), AresError> {
    let project_path = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());

    let store_path = std::path::PathBuf::from(&project_path).join(".ares.db");
    let store = Store::open(std::path::Path::new(&store_path))?;
    
    let governance = ares_governance::GovernanceFacade::new(
        store.clone(),
        std::path::PathBuf::from(&project_path)
    );

    let exemptions = governance.get_exemptions().await.unwrap_or_default();

    println!("ARES Active Policy Exemptions");
    println!("------------------------------------------------------------");
    
    if exemptions.is_empty() {
        println!("No active exemptions found.");
    } else {
        for ex in exemptions {
            println!("ID:          {}", ex.id);
            println!("Reason:      {}", ex.reason);
            println!("Approved By: {}", ex.approved_by);
            println!("Approved At: {}", ex.approved_at);
            println!("Expires At:  {}", ex.expires_at);
            
            if !ex.target_rules.is_empty() {
                println!("Rules:       {}", ex.target_rules.join(", "));
            }
            if !ex.target_nodes.is_empty() {
                println!("Nodes:       {}", ex.target_nodes.join(", "));
            }
            println!("------------------------------------------------------------");
        }
    }

    Ok(())
}

pub async fn execute_pr_check(base_report_path: Option<String>) -> Result<(), AresError> {
    let project_path = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());

    // 1. Generate Head Snapshot
    let store_path = std::path::PathBuf::from(&project_path).join(".ares.db");
    let store = std::sync::Arc::new(ares_store::db::Store::open(std::path::Path::new(&store_path))?);
    
    let kg_store = ares_knowledge_graph::store::KnowledgeGraphStore::new(store.clone());
    let head_graph = kg_store.export_graph()?;
    
    let governance = ares_governance::GovernanceFacade::new(
        (*store).clone(),
        std::path::PathBuf::from(&project_path)
    );
    let project_id = ares_core::ProjectId::from("TEST");
    
    let head_compliance = governance.evaluate_project(&project_id).await.unwrap_or_else(|_| vec![]);
    
    let head_scorecard = governance.get_scorecard(&project_id).await.unwrap_or_else(|_| ares_governance::models::GovernanceScorecard {
        ownership_score: 0.0,
        traceability_score: 0.0,
        evidence_score: 0.0,
        approval_score: 0.0,
        retention_score: 0.0,
        security_score: 0.0,
        architecture_score: 0.0,
        overall_score: 0.0,
    });

    let head_snapshot = ares_pr_engine::models::MemorySnapshot {
        graph: head_graph,
        compliance: head_compliance,
        scorecard: head_scorecard,
    };

    // 2. Load Base Snapshot
    let mut base_snapshot = None;
    if let Some(path) = &base_report_path {
        let content = std::fs::read_to_string(path).map_err(|e| AresError::Io(e))?;
        base_snapshot = Some(serde_json::from_str::<ares_pr_engine::models::MemorySnapshot>(&content)
            .map_err(|e| AresError::Serialization(e.to_string()))?);
    } else {
        return Err(AresError::validation("No base report provided. PR Check requires a baseline for graph delta in CI/CD environments.\nPlease provide a baseline via `--base-report` or ensure a latest certified snapshot exists."));
    }

    let base_snapshot = base_snapshot.expect("Handled above");

    // 3. Evaluate
    let mut readiness = ares_pr_engine::engines::PullRequestEvaluator::evaluate(&base_snapshot, &head_snapshot)?;
    if base_report_path.is_none() {
        readiness.impact.baseline_source = "latest_certified_snapshot".to_string();
    }

    // 4. Output Result
    println!("ARES PR Review\n");
    println!("Memory Impact: {:?}\n", readiness.impact.risk_level);
    
    println!("Requirements Affected: {}", readiness.impact.requirements_affected);
    println!("Decisions Affected: {}", readiness.impact.decisions_affected);
    println!("Traceability Links Removed: {}\n", readiness.impact.traceability_links_removed);
    
    println!("New Violations:");
    if readiness.impact.new_violations_list.is_empty() {
        println!("- None");
    } else {
        for v in &readiness.impact.new_violations_list {
            println!("- {} ({})", v.reason, v.policy_name);
        }
    }
    println!("");
    
    println!("Resolved Violations:");
    if readiness.impact.resolved_violations_list.is_empty() {
        println!("- None");
    } else {
        for v in &readiness.impact.resolved_violations_list {
            println!("- {} ({})", v.reason, v.policy_name);
        }
    }
    println!("");

    println!("Merge Readiness:");
    if readiness.ready {
        println!("READY");
    } else {
        println!("BLOCKED");
        std::process::exit(1);
    }

    Ok(())
}
