use std::sync::Arc;
use ares_core::AresError;
use ares_store::db::Store;
use ares_memory_intelligence::assembler::MemoryContextAssembler;
use ares_validation::validation_runner::ValidationRunner;

pub async fn execute_validate(strict: bool, json: bool, sarif: bool, ci: bool) -> Result<(), AresError> {
    let is_json = json || ci;
    let is_strict = strict || ci;

    // Determine project path
    let project_path = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());

    // Initialize Store
    let store_path = std::path::PathBuf::from(&project_path).join(".ares.db");
    let store = Arc::new(Store::open(std::path::Path::new(&store_path))?);
    let assembler = Arc::new(MemoryContextAssembler::default_from_store((*store).clone()));
    
    let validation_runner = ValidationRunner::new(store, assembler);
    let report = validation_runner.run_certification().await?;

    if sarif {
        // Evaluate project again just to get the violations for SARIF
        // Ideally the ValidationRunner would return the ComplianceResults, but we can quickly fetch it:
        let governance = ares_governance::GovernanceFacade::new(
            (**validation_runner.store()).clone(),
            std::path::PathBuf::from(&project_path)
        );
        let project_id = ares_core::ProjectId::from("TEST");
        let results = governance.evaluate_project(&project_id).await.unwrap_or_default();
        let sarif_json = ares_governance::sarif::GovernanceSarifExporter::export_results(&results);
        let sarif_path = std::path::PathBuf::from(&project_path).join("governance.sarif");
        std::fs::write(&sarif_path, serde_json::to_string_pretty(&sarif_json).unwrap())
            .map_err(|e| AresError::Io(e))?;
        if !is_json {
            println!("Exported SARIF to {:?}", sarif_path);
        }
    }

    if is_json {
        let out = serde_json::to_string_pretty(&report).unwrap();
        println!("{}", out);
    } else {
        println!("ARES MemoryOS Validation Report\n");
        println!("Repository Health:      {:.1}", report.repository_health);
        println!("Memory Health:          {:.1}", report.memory_health);
        println!("Knowledge Debt:         {:.1}\n", report.knowledge_debt);
        
        println!("Traceability Coverage:  {:.0}%", report.traceability_coverage * 100.0);
        println!("Decision Coverage:      {:.0}%", report.decision_coverage * 100.0);
        println!("Evolution Coverage:     {:.0}%\n", report.evolution_coverage * 100.0);

        println!("Canonical Questions Passed: {}/{}", report.canonical_questions_passed, report.total_questions);
        println!("Replay Safety:            {}", if report.replay_safe { "Passed" } else { "Failed" });
        println!("Graph Integrity:          {}\n", if report.graph_integrity_passed { "Passed" } else { "Failed" });

        println!("Governance Level:         {:?}", report.certification_level);
        if let Some(ref d) = report.policy_drift {
            if d.drift_detected {
                println!("Policy Drift Detected!    (Outdated policies: {})", d.outdated_policies.len());
            }
        }
        if let Some(ref e) = report.enforcement {
            if !e.ready {
                println!("Enforcement Readiness:    Failed ({} blocking violations)", e.blocking_violations);
            }
        }

        println!("Memory Certification:     {}", if report.certified { "CERTIFIED" } else { "NOT CERTIFIED" });
    }

    // Exit with 1 if blocking violations exist and strict is enabled
    if is_strict {
        let has_blocking = report.enforcement.map(|e| !e.ready).unwrap_or(false);
        if has_blocking {
            std::process::exit(1);
        }
    }

    Ok(())
}

pub async fn execute_export(out_path: &String) -> Result<(), AresError> {
    let project_path = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());

    let store_path = std::path::PathBuf::from(&project_path).join(".ares.db");
    let store = Arc::new(Store::open(std::path::Path::new(&store_path))?);
    
    // Get Graph
    let kg_store = ares_knowledge_graph::store::KnowledgeGraphStore::new(store.clone());
    let graph = kg_store.export_graph()?;
    
    // Get Compliance & Scorecard
    let governance = ares_governance::GovernanceFacade::new(
        (*store).clone(),
        std::path::PathBuf::from(&project_path)
    );
    let project_id = ares_core::ProjectId::from("TEST");
    let compliance = governance.evaluate_project(&project_id).await.unwrap_or_else(|_| vec![]);
    
    let scorecard = governance.get_scorecard(&project_id).await.unwrap_or_else(|_| ares_governance::models::GovernanceScorecard {
        ownership_score: 0.0,
        traceability_score: 0.0,
        evidence_score: 0.0,
        approval_score: 0.0,
        retention_score: 0.0,
        security_score: 0.0,
        architecture_score: 0.0,
        overall_score: 0.0,
    });

    let snapshot = ares_pr_engine::models::MemorySnapshot {
        graph,
        compliance,
        scorecard,
    };

    let json_out = serde_json::to_string_pretty(&snapshot).unwrap();
    std::fs::write(out_path, json_out).map_err(|e| AresError::Io(e))?;

    println!("Exported Memory Snapshot to {}", out_path);
    Ok(())
}
