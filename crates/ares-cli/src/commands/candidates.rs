use ares_candidates::{CandidateRepository, CandidateStatus, CandidateType, CandidatePromotion};
use ares_core::{AresError, GraphNode, NodeType, NodeId, ProjectId};
use ares_core::types::event::now_micros;
use ares_store::{db::Store, repositories::candidate::SqliteCandidateRepository};
use std::path::PathBuf;
use uuid::Uuid;

fn get_repo() -> Result<SqliteCandidateRepository, AresError> {
    let db_path = PathBuf::from(".ares/ares.db");
    if !db_path.exists() {
        return Err(AresError::validation("Memory database not found. Run `ares ingest .` first."));
    }
    let store = Store::open(&db_path)?;
    Ok(SqliteCandidateRepository::new(store))
}

pub async fn execute_list() -> Result<(), AresError> {
    let repo = get_repo()?;
    // Assuming a single default project ID for now, as CLI runs per repo
    let project_id = "TEST";
    let candidates = repo.list_candidates(project_id, 100, 0).await
        .map_err(|e| AresError::validation(e))?;

    if candidates.is_empty() {
        println!("No pending candidates found.");
        return Ok(());
    }

    println!("{:<40} | {:<15} | {:<12} | {:<5}", "ID", "Type", "Status", "Ev.");
    println!("{:-<40}-+-{:-<15}-+-{:-<12}-+-{:-<5}", "", "", "", "");

    for c in candidates {
        let t_str = match c.candidate_type {
            CandidateType::Requirement => "Requirement",
            CandidateType::Decision => "Decision",
            CandidateType::Architecture => "Architecture",
            CandidateType::Traceability => "Traceability",
        };
        let s_str = match c.status {
            CandidateStatus::Proposed => "Proposed",
            CandidateStatus::UnderReview => "UnderReview",
            CandidateStatus::Approved => "Approved",
            CandidateStatus::Rejected => "Rejected",
            CandidateStatus::Superseded => "Superseded",
        };

        println!("{:<40} | {:<15} | {:<12} | {:<5}", c.id, t_str, s_str, c.confidence.evidence_count);
    }

    Ok(())
}

pub async fn execute_show(id: String) -> Result<(), AresError> {
    let repo = get_repo()?;
    let candidate = repo.get_candidate(&id).await
        .map_err(|e| AresError::validation(e))?
        .ok_or_else(|| AresError::validation("Candidate not found"))?;

    let sources = repo.get_sources(&id).await
        .map_err(|e| AresError::validation(e))?;

    println!("Candidate: {}", candidate.id);
    println!("Title: {}", candidate.title);
    println!("Description: {}", candidate.description);
    
    let t_str = match candidate.candidate_type {
        CandidateType::Requirement => "Requirement",
        CandidateType::Decision => "Decision",
        CandidateType::Architecture => "Architecture",
        CandidateType::Traceability => "Traceability",
    };
    println!("Type: {}", t_str);

    let s_str = match candidate.status {
        CandidateStatus::Proposed => "Proposed",
        CandidateStatus::UnderReview => "UnderReview",
        CandidateStatus::Approved => "Approved",
        CandidateStatus::Rejected => "Rejected",
        CandidateStatus::Superseded => "Superseded",
    };
    println!("Status: {}", s_str);
    println!("Confidence: ({} sources, {} diversity)", 
        candidate.confidence.evidence_count,
        candidate.confidence.source_diversity
    );
    
    println!("\nSources:");
    for s in sources {
        println!("  - [{}] {}", s.source_type, s.source_id);
    }

    Ok(())
}

pub async fn execute_accept(id: String) -> Result<(), AresError> {
    let repo = get_repo()?;
    let candidate = repo.get_candidate(&id).await
        .map_err(|e| AresError::validation(e))?
        .ok_or_else(|| AresError::validation("Candidate not found"))?;

    if candidate.status == CandidateStatus::Approved {
        println!("Candidate {} is already approved.", id);
        return Ok(());
    }

    let node_type = match candidate.candidate_type {
        CandidateType::Requirement => NodeType::Requirement,
        CandidateType::Decision => NodeType::Decision,
        CandidateType::Architecture => NodeType::Concept,
        CandidateType::Traceability => NodeType::Feature,
    };

    let node_id = NodeId::from(format!("node:{}", Uuid::new_v4()));
    let now = now_micros();

    let node = GraphNode {
        id: node_id.clone(),
        project_id: ProjectId::from(candidate.project_id.clone()),
        node_type,
        label: candidate.title.clone(),
        properties: serde_json::json!({
            "description": candidate.description,
            "candidate_id": candidate.id,
        }),
        file_path: None,
        created_at: now,
        updated_at: now,
        deleted_at: None,
    };

    let promotion = CandidatePromotion {
        id: Uuid::new_v4().to_string(),
        candidate_id: candidate.id.clone(),
        promoted_node_id: node_id.clone(),
        promoted_by: "cli_user".to_string(), // In the future, read authenticated user
        promoted_at: now,
        promotion_reason: Some("Manually accepted via CLI".to_string()),
    };

    repo.promote_candidate(&candidate, &promotion, &node, &[])
        .await
        .map_err(|e| AresError::validation(format!("Promotion failed: {}", e)))?;

    println!("Successfully accepted candidate {} and promoted to authoritative node {}.", id, node_id);
    Ok(())
}

pub async fn execute_reject(id: String) -> Result<(), AresError> {
    let repo = get_repo()?;
    let mut candidate = repo.get_candidate(&id).await
        .map_err(|e| AresError::validation(e))?
        .ok_or_else(|| AresError::validation("Candidate not found"))?;

    candidate.status = CandidateStatus::Rejected;
    candidate.updated_at = now_micros();

    repo.update_candidate(&candidate).await
        .map_err(|e| AresError::validation(e))?;

    println!("Successfully rejected candidate {}.", id);
    Ok(())
}
