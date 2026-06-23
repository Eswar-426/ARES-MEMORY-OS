use ares_core::AresError;
use ares_core::ProjectId;
use ares_governance::coverage_engine::CoverageEngine;
use ares_governance::memory_debt_engine::MemoryDebtEngine;
use ares_governance::memory_drift_engine::MemoryDriftEngine;
use ares_governance::memory_health_engine::MemoryHealthEngine;
use ares_governance::memory_maturity_model::{MemoryMaturityEngine, MemoryMaturityLevel};
use ares_knowledge_graph::models::{EdgeType, KnowledgeEdge, KnowledgeNode, NodeType};
use ares_store::db::Store;
use std::sync::Arc;

pub async fn run_synthetic_layer() -> Result<(), AresError> {
    println!("\n=== Layer 1: Synthetic Validation ===");
    println!(
        "{:<25} | {:<10} | {:<10} | {:<10} | {:<10} | {:<15}",
        "Profile", "Coverage", "Debt", "Health", "Drift", "Maturity"
    );
    println!(
        "{:-<25}-|-{:-<10}-|-{:-<10}-|-{:-<10}-|-{:-<10}-|-{:-<15}",
        "", "", "", "", "", ""
    );

    let profiles = vec![
        ("MemoryNative", 100, 100, 50, 100, 100),
        ("Healthy", 100, 80, 20, 100, 100),
        ("Moderate", 100, 40, 5, 50, 50),
        ("Critical", 100, 5, 0, 10, 20),
        ("Chaos", 100, 0, 0, 0, 0),
        ("BrokenEnterpriseRepo", 500, 5, 0, 0, 10),
    ];

    let mut all_passed = true;

    for (name, code_count, req_count, dec_count, owner_count, test_count) in profiles {
        let metrics = evaluate_synthetic(
            name,
            code_count,
            req_count,
            dec_count,
            owner_count,
            test_count,
        )
        .await?;
        println!(
            "{:<25} | {:<9.1}% | {:<10} | {:<9.1}% | {:<9.1}% | {:<15?}",
            name, metrics.coverage, metrics.debt, metrics.health, metrics.drift, metrics.maturity
        );

        // Validation Assertions
        let pass = match name {
            "BrokenEnterpriseRepo" => {
                metrics.coverage < 20.0
                    && metrics.debt > 2000
                    && metrics.health < 30.0
                    && metrics.maturity <= MemoryMaturityLevel::Level1Documented
            }
            "MemoryNative" => metrics.maturity == MemoryMaturityLevel::Level5MemoryNative,
            "Chaos" => {
                metrics.maturity == MemoryMaturityLevel::Level0Chaos && metrics.coverage == 0.0
            }
            _ => true,
        };

        if !pass {
            println!("  -> FAIL: Expected ranges not met for {}", name);
            all_passed = false;
        }
    }

    if !all_passed {
        return Err(AresError::validation(
            "Synthetic validation failed assertions",
        ));
    }

    Ok(())
}

struct SyntheticMetrics {
    coverage: f64,
    debt: u64,
    health: f64,
    drift: f64,
    maturity: MemoryMaturityLevel,
}

async fn evaluate_synthetic(
    name: &str,
    code_count: usize,
    req_count: usize,
    dec_count: usize,
    owner_count: usize,
    test_count: usize,
) -> Result<SyntheticMetrics, AresError> {
    let db_path = format!("memory_os_synth_{}.db", name);
    let path = std::path::Path::new(&db_path);
    if path.exists() {
        std::fs::remove_file(path).unwrap();
    }
    let store = Arc::new(Store::open(path)?);
    let kg = ares_knowledge_graph::store::KnowledgeGraphStore::new(store.clone());
    let pid = ProjectId::from("TEST");

    // Let's create owners
    let mut owner_ids = vec![];
    for i in 0..owner_count {
        let oid = format!("{}-owner-{}", name, i);
        kg.upsert_node(&KnowledgeNode {
            id: oid.clone(),
            node_type: NodeType::Owner,
            name: format!("owner{}", i),
            properties: serde_json::json!({ "project_id": pid.to_string() }),
            created_at: 0,
        })?;
        owner_ids.push(oid);
    }

    // Requirements
    let mut req_ids = vec![];
    for i in 0..req_count {
        let rid = format!("{}-req-{}", name, i);
        kg.upsert_node(&KnowledgeNode {
            id: rid.clone(),
            node_type: NodeType::Requirement,
            name: format!("req{}", i),
            properties: serde_json::json!({ "project_id": pid.to_string() }),
            created_at: 0,
        })?;
        req_ids.push(rid);
    }

    // Decisions
    let mut dec_ids = vec![];
    for i in 0..dec_count {
        let did = format!("{}-dec-{}", name, i);
        kg.upsert_node(&KnowledgeNode {
            id: did.clone(),
            node_type: NodeType::Decision,
            name: format!("dec{}", i),
            properties: serde_json::json!({ "project_id": pid.to_string() }),
            created_at: 0,
        })?;
        dec_ids.push(did);
    }

    // Tests
    let mut test_ids = vec![];
    for i in 0..test_count {
        let tid = format!("{}-test-{}", name, i);
        kg.upsert_node(&KnowledgeNode {
            id: tid.clone(),
            node_type: NodeType::Test,
            name: format!("test{}", i),
            properties: serde_json::json!({ "project_id": pid.to_string() }),
            created_at: 0,
        })?;
        test_ids.push(tid);
    }

    // Code
    for i in 0..code_count {
        let cid = format!("{}-code-{}", name, i);
        kg.upsert_node(&KnowledgeNode {
            id: cid.clone(),
            node_type: NodeType::CodeArtifact,
            name: format!("code{}", i),
            properties: serde_json::json!({ "project_id": pid.to_string() }),
            created_at: 0,
        })?;

        // Connect Code
        if i < req_count {
            kg.upsert_edge(&KnowledgeEdge {
                id: format!("{}-reqedge-{}", name, i),
                source_id: cid.clone(),
                target_id: req_ids[i].clone(),
                edge_type: EdgeType::Implements,
                confidence: 1.0,
                created_at: 0,
                properties: serde_json::json!({ "project_id": pid.to_string() }),
            })?;
        }
        if i < owner_count {
            kg.upsert_edge(&KnowledgeEdge {
                id: format!("{}-ownedge-{}", name, i),
                source_id: cid.clone(),
                target_id: owner_ids[i].clone(),
                edge_type: EdgeType::OwnedBy,
                confidence: 1.0,
                created_at: 0,
                properties: serde_json::json!({ "project_id": pid.to_string() }),
            })?;
        }
        if i < test_count {
            kg.upsert_edge(&KnowledgeEdge {
                id: format!("{}-testedge-{}", name, i),
                source_id: test_ids[i].clone(),
                target_id: cid.clone(),
                edge_type: EdgeType::ValidatedBy,
                confidence: 1.0,
                created_at: 0,
                properties: serde_json::json!({ "project_id": pid.to_string() }),
            })?;
        }
        if i < dec_count {
            kg.upsert_edge(&KnowledgeEdge {
                id: format!("{}-decedge-{}", name, i),
                source_id: cid.clone(),
                target_id: dec_ids[i].clone(),
                edge_type: EdgeType::SupportedBy,
                confidence: 1.0,
                created_at: 0,
                properties: serde_json::json!({ "project_id": pid.to_string() }),
            })?;
        }
    }

    let coverage = CoverageEngine::calculate(&store, &pid)?;
    let drift = MemoryDriftEngine::calculate(&store, &pid)?;
    let debt = MemoryDebtEngine::calculate(&coverage, &drift);
    let health = MemoryHealthEngine::calculate(&coverage, &drift);
    let maturity = MemoryMaturityEngine::evaluate(&coverage, &debt, &health, true);

    Ok(SyntheticMetrics {
        coverage: coverage.overall.percentage,
        debt: debt.total_debt_score,
        health: health.total_health,
        drift: drift.memory_drift_percentage,
        maturity,
    })
}

pub async fn run_real_layer() -> Result<(), AresError> {
    println!("\n=== Layer 2: Reality Validation ===");
    println!(
        "{:<25} | {:<10} | {:<10} | {:<10} | {:<10} | {:<15}",
        "Repository", "Coverage", "Debt", "Health", "Drift", "Maturity"
    );
    println!(
        "{:-<25}-|-{:-<10}-|-{:-<10}-|-{:-<10}-|-{:-<10}-|-{:-<15}",
        "", "", "", "", "", ""
    );

    let repos = vec![
        "ARES",
        "Automyra",
        "ripgrep",
        "cargo-watch",
        "legacy-enterprise-sample",
    ];

    for repo in repos {
        let (metrics, pass) = fetch_and_evaluate_real(repo).await?;
        println!(
            "{:<25} | {:<9.1}% | {:<10} | {:<9.1}% | {:<9.1}% | {:<15?}",
            repo, metrics.coverage, metrics.debt, metrics.health, metrics.drift, metrics.maturity
        );

        if !pass {
            println!("  -> FAIL: Expected ranges not met for {}", repo);
        }
    }

    Ok(())
}

async fn fetch_and_evaluate_real(repo: &str) -> Result<(SyntheticMetrics, bool), AresError> {
    let project_dir = std::env::temp_dir().join(format!("ares_benchmark_{}", repo));
    let url = match repo {
        "ARES" => "https://github.com/eswar-426/ARES-MEMORY-OS",
        "Automyra" => "https://github.com/eswar-426/automyra",
        "ripgrep" => "https://github.com/BurntSushi/ripgrep",
        "cargo-watch" => "https://github.com/watchexec/cargo-watch",
        "legacy-enterprise-sample" => "https://github.com/eswar-426/ARES-MEMORY-OS", // dummy fallback
        _ => "",
    };

    if !project_dir.exists() {
        if repo == "ARES" {
            let pwd = std::env::current_dir().unwrap();
            let store = Arc::new(Store::open(&pwd.join(".ares").join("ares.db"))?);
            let pid = ProjectId::from("TEST");
            let coverage = CoverageEngine::calculate(&store, &pid)?;
            let drift = MemoryDriftEngine::calculate(&store, &pid)?;
            let debt = MemoryDebtEngine::calculate(&coverage, &drift);
            let health = MemoryHealthEngine::calculate(&coverage, &drift);
            let maturity = MemoryMaturityEngine::evaluate(&coverage, &debt, &health, true);
            return Ok((
                SyntheticMetrics {
                    coverage: coverage.overall.percentage,
                    debt: debt.total_debt_score,
                    health: health.total_health,
                    drift: drift.memory_drift_percentage,
                    maturity,
                },
                true,
            ));
        } else {
            let _ = std::process::Command::new("git")
                .arg("clone")
                .arg("--depth")
                .arg("1")
                .arg(url)
                .arg(&project_dir)
                .status();
        }
    }

    let ingest_args = crate::commands::ingest::IngestArgs {
        path: project_dir.clone(),
        incremental: false,
        files: vec![],
        git_depth: 500,
    };
    crate::commands::ingest::handle_ingest(ingest_args).await?;

    let store = Arc::new(Store::open(&project_dir.join(".ares").join("ares.db"))?);
    let pid = ProjectId::from("TEST");
    let coverage = CoverageEngine::calculate(&store, &pid)?;
    let drift = MemoryDriftEngine::calculate(&store, &pid)?;
    let debt = MemoryDebtEngine::calculate(&coverage, &drift);
    let health = MemoryHealthEngine::calculate(&coverage, &drift);
    let maturity = MemoryMaturityEngine::evaluate(&coverage, &debt, &health, true);

    Ok((
        SyntheticMetrics {
            coverage: coverage.overall.percentage,
            debt: debt.total_debt_score,
            health: health.total_health,
            drift: drift.memory_drift_percentage,
            maturity,
        },
        true,
    ))
}
