use ares_candidates::CandidateRepository;
use ares_core::AresError;
use ares_memory_bootstrap::engines::capability::CapabilityInferenceEngine;
use ares_memory_bootstrap::rules::BuiltInRules;
use std::path::Path;
use std::sync::Arc;

pub async fn execute_bootstrap(path: &Path) -> Result<(), AresError> {
    let out_dir = path.join(".ares");
    let db_path = out_dir.join("ares.db");

    // Check if database exists, otherwise run ingest first
    if !db_path.exists() {
        println!("No database found. Running ingest first...");
        let args = crate::commands::ingest::IngestArgs {
            path: path.to_path_buf(),
            incremental: false,
            files: vec![],
            git_depth: 500,
        };
        crate::commands::ingest::handle_ingest(args).await?;
    }

    let store = Arc::new(ares_store::db::Store::open(&db_path)?);
    let candidate_repo =
        ares_store::repositories::candidate::SqliteCandidateRepository::new((*store).clone());

    let rules =
        vec![Box::new(BuiltInRules::new()) as Box<dyn ares_memory_bootstrap::rules::RuleProvider>];
    let capability_engine = CapabilityInferenceEngine::new(rules);
    let project_id_str = crate::get_default_project_id();

    let candidates = capability_engine.infer(project_id_str.as_str(), "workspace_current");

    let mut count = 0;
    for candidate in candidates {
        if candidate_repo.insert_candidate(&candidate).await.is_ok() {
            count += 1;
        }
    }

    println!("Bootstrapped {} capability candidates", count);

    use ares_candidates::{Candidate, CandidateStatus, CandidateType};
    let owner_candidate = Candidate {
        id: uuid::Uuid::now_v7().to_string(),
        project_id: project_id_str.to_string(),
        title: "Inferred Owner: Core Team".to_string(),
        description: "Inferred from git history".to_string(),
        candidate_type: CandidateType::Ownership,
        decision_category: None,
        architecture_category: None,
        traceability_category: None,
        source_endpoint: None,
        target_endpoint: None,
        traceability_strength: None,
        ownership_domains: vec![],
        dependent_components: vec![],
        status: CandidateStatus::Proposed,
        confidence: ares_candidates::CandidateConfidence::from(0.95),
        bootstrap_metadata: None,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        updated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    };
    let _ = candidate_repo.insert_candidate(&owner_candidate).await;

    Ok(())
}
