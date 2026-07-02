use ares_core::AresError;
use ares_store::db::Store;

pub async fn execute_exemptions() -> Result<(), AresError> {
    let project_path = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());

    let store_path = std::path::PathBuf::from(&project_path)
        .join(".ares")
        .join("ares.db");
    let store = Store::open(std::path::Path::new(&store_path))?;

    let governance = ares_governance::GovernanceFacade::new(
        store.clone(),
        std::path::PathBuf::from(&project_path),
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
    let store_path = std::path::PathBuf::from(&project_path)
        .join(".ares")
        .join("ares.db");
    let store = std::sync::Arc::new(ares_store::db::Store::open(std::path::Path::new(
        &store_path,
    ))?);

    let kg_store = ares_knowledge_graph::store::KnowledgeGraphStore::new(store.clone());
    let head_graph = kg_store.export_graph()?;

    let governance = ares_governance::GovernanceFacade::new(
        (*store).clone(),
        std::path::PathBuf::from(&project_path),
    );
    let project_id = crate::get_default_project_id();

    let head_compliance = governance
        .evaluate_project(&project_id)
        .await
        .unwrap_or_else(|_| vec![]);

    let head_scorecard = governance.get_scorecard(&project_id).await.unwrap_or(
        ares_governance::models::GovernanceScorecard {
            ownership_score: 0.0,
            traceability_score: 0.0,
            evidence_score: 0.0,
            approval_score: 0.0,
            retention_score: 0.0,
            security_score: 0.0,
            architecture_score: 0.0,
            overall_score: 0.0,
        },
    );

    let head_snapshot = ares_pr_engine::models::MemorySnapshot {
        graph: head_graph,
        compliance: head_compliance,
        scorecard: head_scorecard,
    };

    // 2. Load Base Snapshot
    let mut base_snapshot = None;
    if let Some(path) = &base_report_path {
        let content = std::fs::read_to_string(path).map_err(AresError::Io)?;
        base_snapshot = Some(
            serde_json::from_str::<ares_pr_engine::models::MemorySnapshot>(&content)
                .map_err(|e| AresError::Serialization(e.to_string()))?,
        );
    } else {
        return Err(AresError::validation("No base report provided. PR Check requires a baseline for graph delta in CI/CD environments.\nPlease provide a baseline via `--base-report` or ensure a latest certified snapshot exists."));
    }

    let base_snapshot = base_snapshot.expect("Handled above");

    // 3. Evaluate
    let mut readiness =
        ares_pr_engine::engines::PullRequestEvaluator::evaluate(&base_snapshot, &head_snapshot)?;
    if base_report_path.is_none() {
        readiness.impact.baseline_source = "latest_certified_snapshot".to_string();
    }

    // 4. Output Result
    println!("ARES PR Review\n");
    println!("Memory Impact: {:?}\n", readiness.impact.risk_level);

    println!(
        "Requirements Affected: {}",
        readiness.impact.requirements_affected
    );
    println!(
        "Decisions Affected: {}",
        readiness.impact.decisions_affected
    );
    println!(
        "Traceability Links Removed: {}\n",
        readiness.impact.traceability_links_removed
    );

    println!("New Violations:");
    if readiness.impact.new_violations_list.is_empty() {
        println!("- None");
    } else {
        for v in &readiness.impact.new_violations_list {
            println!("- {} ({})", v.reason, v.policy_name);
        }
    }
    println!();

    println!("Resolved Violations:");
    if readiness.impact.resolved_violations_list.is_empty() {
        println!("- None");
    } else {
        for v in &readiness.impact.resolved_violations_list {
            println!("- {} ({})", v.reason, v.policy_name);
        }
    }
    println!();

    println!("Merge Readiness:");
    if readiness.ready {
        println!("READY");
    } else {
        println!("BLOCKED");
        std::process::exit(1);
    }

    Ok(())
}

pub async fn execute_report(json: bool, markdown: bool) -> Result<(), AresError> {
    println!("Generating full MemoryOS Governance Report...");
    execute_coverage(json, markdown).await?;
    execute_debt(json, markdown).await?;
    execute_health(json, markdown).await?;
    execute_maturity(json, markdown).await?;
    execute_drift(json, markdown).await?;
    execute_confidence(json, markdown).await?;
    Ok(())
}

pub async fn execute_coverage(json: bool, markdown: bool) -> Result<(), AresError> {
    let project_path = std::env::current_dir().unwrap_or_default();
    let store_path = project_path.join(".ares").join("ares.db");
    if !store_path.exists() {
        return Err(AresError::validation("No repository memory found."));
    }

    let store = std::sync::Arc::new(ares_store::db::Store::open(&store_path)?);
    let project_id = crate::get_default_project_id();
    let metrics = ares_governance::coverage_engine::CoverageEngine::calculate(&store, &project_id)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&metrics).unwrap());
    } else if markdown {
        let md = format!("# Coverage Report\n\nCapture Rate: {:.2}%\nOverall Coverage: {:.2}%\nRequirements: {:.2}%\nCode: {:.2}%\n", metrics.capture_rate.rate, metrics.overall.percentage, metrics.requirements.percentage, metrics.overall.percentage);
        std::fs::write("coverage_report.md", md).unwrap();
        println!("Generated coverage_report.md");
    } else {
        println!(
            "Memory Capture Rate: {:.2}% ({}/{} Sources)",
            metrics.capture_rate.rate,
            metrics.capture_rate.captured_sources,
            metrics.capture_rate.available_sources
        );
        println!(
            "  - Git Commits: {}",
            if metrics.capture_rate.git_commits {
                "Yes"
            } else {
                "No"
            }
        );
        println!(
            "  - Git Releases: {}",
            if metrics.capture_rate.git_releases {
                "Yes"
            } else {
                "No"
            }
        );
        println!(
            "  - Git Blame: {}",
            if metrics.capture_rate.git_blame {
                "Yes"
            } else {
                "No"
            }
        );
        println!(
            "  - CODEOWNERS: {}",
            if metrics.capture_rate.codeowners {
                "Yes"
            } else {
                "No"
            }
        );
        println!("-----------------------------------");
        println!("Memory Coverage: {:.2}%", metrics.overall.percentage);
    }
    Ok(())
}

pub async fn execute_debt(json: bool, markdown: bool) -> Result<(), AresError> {
    let project_path = std::env::current_dir().unwrap_or_default();
    let store_path = project_path.join(".ares").join("ares.db");
    let store = std::sync::Arc::new(ares_store::db::Store::open(&store_path)?);
    let project_id = crate::get_default_project_id();

    let coverage =
        ares_governance::coverage_engine::CoverageEngine::calculate(&store, &project_id)?;
    let drift =
        ares_governance::memory_drift_engine::MemoryDriftEngine::calculate(&store, &project_id)?;

    let metrics =
        ares_governance::memory_debt_engine::MemoryDebtEngine::calculate(&coverage, &drift);

    if json {
        println!("{}", serde_json::to_string_pretty(&metrics).unwrap());
    } else if markdown {
        let md = format!(
            "# Memory Debt Report\n\nTotal Debt Score: {}\nSeverity: {:?}\n",
            metrics.total_debt_score, metrics.severity
        );
        std::fs::write("debt_report.md", md).unwrap();
        println!("Generated debt_report.md");
    } else {
        println!("Total Debt Score: {}", metrics.total_debt_score);
    }
    Ok(())
}

pub async fn execute_health(json: bool, markdown: bool) -> Result<(), AresError> {
    let project_path = std::env::current_dir().unwrap_or_default();
    let store_path = project_path.join(".ares").join("ares.db");
    let store = std::sync::Arc::new(ares_store::db::Store::open(&store_path)?);
    let project_id = crate::get_default_project_id();

    let coverage =
        ares_governance::coverage_engine::CoverageEngine::calculate(&store, &project_id)?;

    let drift =
        ares_governance::memory_drift_engine::MemoryDriftEngine::calculate(&store, &project_id)?;

    let score =
        ares_governance::memory_health_engine::MemoryHealthEngine::calculate(&coverage, &drift);

    if json {
        println!("{}", serde_json::to_string_pretty(&score).unwrap());
    } else if markdown {
        let md = format!(
            "# Memory Health Report\n\nTotal Health: {:.2}%\n",
            score.total_health
        );
        std::fs::write("health_report.md", md).unwrap();
        println!("Generated health_report.md");
    } else {
        println!("Health Score: {:.2}%", score.total_health);
    }
    Ok(())
}

pub async fn execute_maturity(json: bool, markdown: bool) -> Result<(), AresError> {
    let project_path = std::env::current_dir().unwrap_or_default();
    let store_path = project_path.join(".ares").join("ares.db");
    let store = std::sync::Arc::new(ares_store::db::Store::open(&store_path)?);
    let project_id = crate::get_default_project_id();

    let coverage =
        ares_governance::coverage_engine::CoverageEngine::calculate(&store, &project_id)?;
    let drift =
        ares_governance::memory_drift_engine::MemoryDriftEngine::calculate(&store, &project_id)?;
    let debt = ares_governance::memory_debt_engine::MemoryDebtEngine::calculate(&coverage, &drift);
    let health =
        ares_governance::memory_health_engine::MemoryHealthEngine::calculate(&coverage, &drift);

    let maturity = ares_governance::memory_maturity_model::MemoryMaturityEngine::evaluate(
        &coverage, &debt, &health, false,
    );

    if json {
        println!("{{\"level\": \"{:?}\"}}", maturity);
    } else if markdown {
        let md = format!("# Memory Maturity Report\n\nLevel: {:?}\n", maturity);
        std::fs::write("maturity_report.md", md).unwrap();
        println!("Generated maturity_report.md");
    } else {
        println!("Maturity Level: {:?}", maturity);
    }
    Ok(())
}

pub async fn execute_drift(json: bool, markdown: bool) -> Result<(), AresError> {
    let project_path = std::env::current_dir().unwrap_or_default();
    let store_path = project_path.join(".ares").join("ares.db");
    let store = std::sync::Arc::new(ares_store::db::Store::open(&store_path)?);
    let project_id = crate::get_default_project_id();

    let drift =
        ares_governance::memory_drift_engine::MemoryDriftEngine::calculate(&store, &project_id)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&drift).unwrap());
    } else if markdown {
        let md = format!(
            "# Memory Drift Report\n\nDrifted Artifacts: {}\n",
            drift.artifacts_changed
        );
        std::fs::write("drift_report.md", md).unwrap();
        println!("Generated drift_report.md");
    } else {
        println!("Drifted Artifacts: {}", drift.artifacts_changed);
    }
    Ok(())
}

pub async fn execute_confidence(_json: bool, _markdown: bool) -> Result<(), AresError> {
    println!("Confidence: Not fully implemented yet");
    Ok(())
}

pub async fn execute_check(baseline: Option<String>) -> Result<(), AresError> {
    println!("ARES Governance Check");

    let base_path =
        baseline.ok_or_else(|| AresError::validation("No baseline provided for check"))?;
    let content = std::fs::read_to_string(&base_path).map_err(AresError::Io)?;
    let base_snapshot: ares_pr_engine::models::MemorySnapshot =
        serde_json::from_str(&content).map_err(|e| AresError::Serialization(e.to_string()))?;

    // Create a temporary store for the baseline
    let temp_db_path =
        std::env::temp_dir().join(format!("ares_baseline_check_{}.db", uuid::Uuid::new_v4()));
    if temp_db_path.exists() {
        let _ = std::fs::remove_file(&temp_db_path);
    }
    let raw_store = std::sync::Arc::new(ares_store::Store::open(&temp_db_path)?);
    let repo = std::sync::Arc::new(ares_store::repositories::graph::SqliteGraphRepository::new(
        (*raw_store).clone(),
    ));

    let project_id = crate::get_default_project_id();

    // Insert dummy project to satisfy foreign key constraint
    raw_store.get_conn()?.execute(
        "INSERT OR IGNORE INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES (?1, ?1, '', '', '', '', 'greenfield', 0, 0)",
        [project_id.as_str()],
    ).map_err(|e| ares_core::AresError::db(e.to_string()))?;

    // Map KnowledgeNode to GraphNode and insert into the baseline
    for node in &base_snapshot.graph.nodes {
        let node_type_str = format!("{:?}", node.node_type);
        let ntype = match node_type_str.as_str() {
            "\"CodeArtifact\"" | "CodeArtifact" => ares_core::NodeType::File,
            "\"Requirement\"" | "Requirement" => ares_core::NodeType::Requirement,
            "\"Decision\"" | "Decision" => ares_core::NodeType::Decision,
            "\"Owner\"" | "Owner" => ares_core::NodeType::Tag,
            _ => node_type_str
                .parse()
                .unwrap_or(ares_core::NodeType::Concept),
        };
        // DEBUG: Print node type mapping
        if ntype == ares_core::NodeType::Requirement {
            println!(
                "DEBUG_MAPPING: REQ node_type_str='{}' -> {:?}",
                node_type_str, ntype
            );
        } else if ntype == ares_core::NodeType::Decision {
            println!(
                "DEBUG_MAPPING: DEC node_type_str='{}' -> {:?}",
                node_type_str, ntype
            );
        } else if ntype == ares_core::NodeType::Concept {
            println!(
                "DEBUG_MAPPING: CON node_type_str='{}' -> {:?}",
                node_type_str, ntype
            );
        }
        let file_path = node
            .properties
            .get("path")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                node.properties
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            });
        let properties_val =
            serde_json::to_value(&node.properties).unwrap_or(serde_json::json!({}));
        let gn = ares_core::GraphNode {
            id: ares_core::NodeId::from(node.id.as_str()),
            project_id: project_id.clone(),
            node_type: ntype,
            label: node.name.clone(),
            properties: properties_val,
            file_path,
            created_at: node.created_at,
            updated_at: node.created_at,
            deleted_at: None,
        };
        if let Err(e) = repo.upsert_node(gn) {
            println!("DEBUG: Failed to upsert node {}: {:?}", node.id, e);
        }
    }
    for edge in &base_snapshot.graph.edges {
        use ares_knowledge_graph::models::EdgeType as KgEdgeType;
        let etype = match &edge.edge_type {
            KgEdgeType::OwnedBy => ares_core::EdgeType::OwnedBy,
            KgEdgeType::Contains => ares_core::EdgeType::Contains,
            KgEdgeType::DependsOn => ares_core::EdgeType::DependsOn,
            KgEdgeType::Implements => ares_core::EdgeType::Implements,
            KgEdgeType::ImplementedBy => ares_core::EdgeType::Implements,
            KgEdgeType::Drives => ares_core::EdgeType::Drives,
            KgEdgeType::SupportedBy => ares_core::EdgeType::SupportedBy,
            KgEdgeType::Supports => ares_core::EdgeType::SupportedBy,
            KgEdgeType::ValidatedBy => ares_core::EdgeType::ValidatedBy,
            KgEdgeType::DerivedFrom => ares_core::EdgeType::DerivedFrom,
            KgEdgeType::Supersedes => ares_core::EdgeType::Supersedes,
            KgEdgeType::References => ares_core::EdgeType::References,
            KgEdgeType::Causes => ares_core::EdgeType::Caused,
            KgEdgeType::Resolves => ares_core::EdgeType::FixedBy,
            _ => ares_core::EdgeType::RelatedTo,
        };
        let ge = ares_core::GraphEdge {
            id: edge.id.clone(),
            project_id: project_id.clone(),
            from_node_id: ares_core::NodeId::from(edge.source_id.as_str()),
            to_node_id: ares_core::NodeId::from(edge.target_id.as_str()),
            edge_type: etype,
            weight: 1.0,
            confidence: edge.confidence,
            source: "scanner".to_string(), // MUST be one of 'human', 'scanner', 'agent', 'inference'
            valid_from: edge.created_at,
            valid_until: None,
            created_at: edge.created_at,
        };
        if let Err(err) = repo.upsert_edge(ge) {
            println!("DEBUG: Failed to upsert edge {}: {:?}", edge.id, err);
        }
    }

    let project_id = crate::get_default_project_id();

    let base_coverage =
        ares_governance::coverage_engine::CoverageEngine::calculate(&raw_store, &project_id)?;
    println!(
        "DEBUG: Base Coverage total decisions: {}",
        base_coverage.decisions.total
    );
    let base_drift = ares_governance::memory_drift_engine::MemoryDriftEngine::calculate(
        &raw_store,
        &project_id,
    )?;
    let base_debt = ares_governance::memory_debt_engine::MemoryDebtEngine::calculate(
        &base_coverage,
        &base_drift,
    );
    let base_health = ares_governance::memory_health_engine::MemoryHealthEngine::calculate(
        &base_coverage,
        &base_drift,
    );

    // Current store
    let project_path = std::env::current_dir().unwrap_or_default();
    let store_path = project_path.join(".ares").join("ares.db");
    let current_store = std::sync::Arc::new(ares_store::db::Store::open(&store_path)?);

    let current_coverage =
        ares_governance::coverage_engine::CoverageEngine::calculate(&current_store, &project_id)?;
    println!("DEBUG: Base Coverage overall: {:?}", base_coverage.overall);
    println!(
        "DEBUG: Current Coverage overall: {:?}",
        current_coverage.overall
    );
    println!(
        "DEBUG: Base Coverage total decisions: {}",
        base_coverage.decisions.total
    );
    println!(
        "DEBUG: Current Coverage total decisions: {}",
        current_coverage.decisions.total
    );
    let current_drift = ares_governance::memory_drift_engine::MemoryDriftEngine::calculate(
        &current_store,
        &project_id,
    )?;
    let current_debt = ares_governance::memory_debt_engine::MemoryDebtEngine::calculate(
        &current_coverage,
        &current_drift,
    );
    let current_health = ares_governance::memory_health_engine::MemoryHealthEngine::calculate(
        &current_coverage,
        &current_drift,
    );

    let status = ares_governance::memory_gatekeeper::MemoryGatekeeper::evaluate_delta(
        &base_coverage,
        &current_coverage,
        &base_debt,
        &current_debt,
        &base_health,
        &current_health,
    );

    match status {
        ares_governance::memory_gatekeeper::GatekeeperStatus::Pass => {
            println!("READY: Governance Checks Passed.");
        }
        ares_governance::memory_gatekeeper::GatekeeperStatus::SoftFail(reasons) => {
            println!("WARNING: Governance SoftFail triggered.");
            for r in reasons {
                println!("  - {}", r);
            }
        }
        ares_governance::memory_gatekeeper::GatekeeperStatus::HardFail(reasons) => {
            println!("BLOCKED: Governance HardFail triggered.");
            for r in reasons {
                println!("  - {}", r);
            }
            std::process::exit(1);
        }
    }

    Ok(())
}

pub async fn execute_snapshot_create(out: String) -> Result<(), AresError> {
    println!("Creating memory snapshot...");
    crate::commands::memory::execute_export(&out).await?;
    println!("Snapshot saved to {}", out);
    Ok(())
}

pub async fn execute_snapshot_compare(baseline: String) -> Result<(), AresError> {
    println!("Comparing current memory against snapshot: {}", baseline);
    execute_pr_check(Some(baseline)).await?;
    Ok(())
}

pub async fn execute_benchmark(synthetic: bool, real: bool, all: bool) -> Result<(), AresError> {
    let mut run_synth = synthetic || all;
    let mut run_real = real || all;
    if !run_synth && !run_real {
        run_synth = true;
        run_real = true;
    }

    if run_synth {
        crate::commands::benchmark::run_synthetic_layer().await?;
    }
    if run_real {
        crate::commands::benchmark::run_real_layer().await?;
    }
    Ok(())
}
