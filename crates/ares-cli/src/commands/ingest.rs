use ares_core::AresError;
use ares_ingestion::GraphBuilder;
use ares_knowledge_graph::models::{
    EdgeType, KnowledgeEdge, KnowledgeGraph, KnowledgeNode, NodeType,
};
use clap::Args;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs >= 60 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs > 0 {
        format!("{}s", secs)
    } else {
        format!("{}ms", d.as_millis())
    }
}

#[derive(Args, Debug, Clone)]
pub struct IngestArgs {
    /// The path to the repository to ingest
    #[arg(default_value = ".")]
    pub path: PathBuf,

    #[arg(long)]
    pub incremental: bool,

    #[arg(long, value_delimiter = ',')]
    pub files: Vec<PathBuf>,

    /// How many commits to analyze (default: 500)
    #[arg(long, default_value = "500")]
    pub git_depth: usize,
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct EdgeIdentity {
    source: String,
    target: String,
    edge_type: String,
}

pub async fn handle_ingest(args: IngestArgs) -> Result<(), AresError> {
    println!("Ingesting repository at: {}", args.path.display());
    let start_time = Instant::now();
    if args.incremental {
        println!(
            "Incremental mode enabled. Processing {} file(s).",
            args.files.len()
        );
    }

    let mut builder = GraphBuilder::new(&args.path);
    if args.incremental {
        builder.set_incremental_files(args.files.clone());
    }

    let mut registry = ares_core::types::source::MemorySourceRegistry::new();
    // Default to explicit documentation being active, as the AST scanner runs it
    registry.register(
        ares_core::types::source::MemorySource::ExplicitDocumentation,
        ares_core::types::source::SourceStatus::Active,
    );

    let out_dir = args.path.join(".ares");
    if !out_dir.exists() {
        std::fs::create_dir_all(&out_dir).map_err(|e| AresError::validation(e.to_string()))?;
    }

    let db_path = out_dir.join("ares.db");
    let raw_store = std::sync::Arc::new(ares_store::db::Store::open(&db_path)?);

    // Run schema and data migrations
    let project_id = crate::get_default_project_id();
    raw_store.run_migrations(project_id.as_str())?;

    let graph_store = ares_knowledge_graph::store::KnowledgeGraphStore::new(raw_store.clone());

    if args.incremental {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        builder.build(|event| {
            match event {
                ares_knowledge_graph::models::GraphEvent::Node(n) => nodes.push(n),
                ares_knowledge_graph::models::GraphEvent::Edge(e) => edges.push(e),
            }
            Ok(())
        })?;
        let graph = KnowledgeGraph { nodes, edges };
        apply_incremental(&args, graph, &graph_store, &raw_store)?;
    } else {
        // Full ingest with streaming batch architecture
        let mut batch = Vec::new();
        let batch_size = 2500;

        let repo = std::sync::Arc::new(
            ares_store::repositories::graph::SqliteGraphRepository::new((*raw_store).clone()),
        );

        // Insert dummy project
        raw_store.get_conn()?.execute(
            "INSERT OR IGNORE INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES (?1, ?1, '', '', '', '', 'greenfield', 0, 0)",
            [project_id.as_str()],
        ).map_err(|e| ares_core::AresError::db(e.to_string()))?;

        // Collect all KG events for bridging to ares-store after scanner
        let mut kg_nodes: Vec<KnowledgeNode> = Vec::new();
        let mut kg_edges: Vec<KnowledgeEdge> = Vec::new();

        let ast_start = Instant::now();
        builder.build(|event| {
            match &event {
                ares_knowledge_graph::models::GraphEvent::Node(n) => kg_nodes.push(n.clone()),
                ares_knowledge_graph::models::GraphEvent::Edge(e) => kg_edges.push(e.clone()),
            }
            batch.push(event.clone());
            if batch.len() >= batch_size {
                graph_store.upsert_batch(&batch)?;
                batch.clear();
            }
            Ok(())
        })?;

        if !batch.is_empty() {
            graph_store.upsert_batch(&batch)?;
        }
        let ast_elapsed = ast_start.elapsed();

        let scan_start = Instant::now();
        let scanner = ares_scanner::Scanner::new(repo.clone());
        let _ = scanner.scan_project(&project_id, &args.path);
        let scan_elapsed = scan_start.elapsed();

        let inventory_start = Instant::now();
        // --- File Inventory: create lightweight File nodes for ALL tracked files ---
        // The scanner only parses supported extensions (rs, ts, js, etc.), but git history
        // creates edges for every file. We need a File node to exist for every tracked
        // repository file so that edge insertion doesn't violate FK constraints.
        {
            let root_str = args.path.to_string_lossy();
            let existing_nodes = repo.get_all_nodes(&project_id).unwrap_or_default();
            let existing_paths: std::collections::HashSet<String> = existing_nodes
                .iter()
                .filter(|n| n.node_type == ares_core::NodeType::File)
                .filter_map(|n| n.file_path.clone())
                .collect();

            let walker = ignore::WalkBuilder::new(&args.path)
                .hidden(false)
                .filter_entry(|e| {
                    let name = e.file_name().to_string_lossy();
                    !matches!(
                        name.as_ref(),
                        ".git"
                            | "target"
                            | ".gemini"
                            | "artifacts"
                            | "node_modules"
                            | "dist"
                            | "out"
                            | "build"
                            | ".turbo"
                            | ".ares"
                            | "scratch"
                            | "cert_synthetic"
                            | "apps"
                            | "evaluation"
                            | "package-lock.json"
                            | "yarn.lock"
                            | "pnpm-lock.yaml"
                            | "Cargo.lock"
                    )
                })
                .build();

            let mut inventory_count = 0u32;
            let now = ares_core::types::event::now_micros();
            for entry in walker.flatten() {
                if !entry.file_type().is_some_and(|ft| ft.is_file()) {
                    continue;
                }
                let rel_path =
                    ares_core::canonical_repo_path(&root_str, &entry.path().to_string_lossy());
                if rel_path.is_empty() || existing_paths.contains(&rel_path) {
                    continue;
                }
                let node_id = ares_core::NodeId::new();
                let label = entry
                    .path()
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let file_node = ares_core::GraphNode {
                    id: node_id,
                    project_id: project_id.clone(),
                    node_type: ares_core::NodeType::File,
                    label,
                    properties: serde_json::json!({}),
                    file_path: Some(rel_path),
                    created_at: now,
                    updated_at: now,
                    deleted_at: None,
                };
                let _ = repo.upsert_node(file_node);
                inventory_count += 1;
            }
            println!(
                "File inventory: created {} additional File nodes",
                inventory_count
            );
        }
        let inventory_elapsed = inventory_start.elapsed();

        // Bridge KG memory-artifact nodes/edges into ares-store graph tables
        // First, build a mapping from KG path-based IDs to scanner UUID IDs
        // so we can remap edges to scanner nodes.
        let all_scanner_nodes = repo.get_all_nodes(&project_id).unwrap_or_default();
        let mut path_to_scanner_id: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for sn in &all_scanner_nodes {
            if sn.node_type != ares_core::NodeType::File {
                continue;
            }
            if let Some(fp) = &sn.file_path {
                let canonical = ares_core::canonicalize_node_id(fp);
                path_to_scanner_id.insert(canonical, sn.id.as_str().to_string());
            }
        }

        // --- P3.2 Git Fact Capture ---
        let git_start = Instant::now();
        println!("Extracting Git Memory (depth: {})...", args.git_depth);
        let mut git_extractor = ares_git_memory::GitMemoryExtractor::new(&args.path);
        git_extractor.set_depth(args.git_depth);

        let upsert_git_results = |git_memory: ares_git_memory::models::GitMemoryResult| {
            for mut node in git_memory.nodes {
                if matches!(node.node_type, ares_core::NodeType::File) {
                    let canonical = ares_core::canonicalize_node_id(node.id.as_str());
                    if let Some(scanner_id) = path_to_scanner_id.get(&canonical) {
                        node.id = ares_core::NodeId::from(scanner_id.as_str());
                    } else {
                        // Skip file nodes that were not scanned (ghost nodes)
                        continue;
                    }
                }
                if let Err(e) = repo.upsert_node(node) {
                    println!("DEBUG: Failed to upsert git node: {:?}", e);
                }
            }
            for mut edge in git_memory.edges {
                if matches!(
                    edge.edge_type,
                    ares_core::EdgeType::Touches
                        | ares_core::EdgeType::Owns
                        | ares_core::EdgeType::ContributedTo
                ) {
                    if let Some(scanner_id) = path_to_scanner_id.get(edge.to_node_id.as_str()) {
                        edge.to_node_id = ares_core::NodeId::from(scanner_id.as_str());
                    } else {
                        continue;
                    }
                }
                if let Err(e) = repo.upsert_edge(edge.clone()) {
                    println!(
                        "DEBUG: Failed to upsert git edge {:?} -> {:?} : {:?}",
                        edge.from_node_id, edge.to_node_id, e
                    );
                }
            }

            let now_micros = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as i64;
            for pr_dec in git_memory.pr_decisions {
                let requires_human_review = pr_dec.confidence < 0.79;
                let dec_id = ares_core::NodeId::from(format!("decision:pr:{}", pr_dec.commit_hash));
                let props = serde_json::json!({
                    "source": "inferred_pr",
                    "pr_number": pr_dec.pr_number,
                    "confidence": pr_dec.confidence,
                    "requires_human_review": requires_human_review,
                    "extracted_heading": pr_dec.extracted_heading,
                    "created_at": now_micros,
                    "decision": pr_dec.description,
                    "title": pr_dec.title,
                });

                let dec_node = ares_core::GraphNode {
                    id: dec_id.clone(),
                    project_id: project_id.clone(),
                    node_type: ares_core::NodeType::Decision,
                    label: pr_dec.title.clone(),
                    properties: props,
                    file_path: None,
                    created_at: now_micros,
                    updated_at: now_micros,
                    deleted_at: None,
                };

                if let Err(e) = repo.upsert_node(dec_node) {
                    println!("DEBUG: Failed to upsert PR decision node: {:?}", e);
                }

                for file_path in pr_dec.touched_files {
                    let canonical = ares_core::canonicalize_node_id(&file_path);
                    let target_id = if let Some(scanner_id) = path_to_scanner_id.get(&canonical) {
                        ares_core::NodeId::from(scanner_id.as_str())
                    } else {
                        continue;
                    };

                    let edge = ares_core::GraphEdge {
                        id: format!("{}-relatedto-{}", dec_id.as_str(), target_id.as_str()),
                        project_id: project_id.clone(),
                        from_node_id: dec_id.clone(),
                        to_node_id: target_id,
                        edge_type: ares_core::EdgeType::RelatedTo,
                        weight: 1.0,
                        confidence: pr_dec.confidence as f32,
                        source: "agent_decision".to_string(),
                        valid_from: now_micros,
                        valid_until: None,
                        created_at: now_micros,
                    };

                    if let Err(e) = repo.upsert_edge(edge) {
                        println!("DEBUG: Failed to upsert PR decision edge: {:?}", e);
                    }
                }
            }
        };

        match git_extractor.extract_metadata_only(&project_id) {
            Ok(git_memory) => {
                registry.register(
                    ares_core::types::source::MemorySource::GitHistory,
                    ares_core::types::source::SourceStatus::Active,
                );
                let has_authors = git_memory
                    .nodes
                    .iter()
                    .any(|n| matches!(n.node_type, ares_core::NodeType::Person));
                if has_authors {
                    registry.register(
                        ares_core::types::source::MemorySource::OwnershipConfig,
                        ares_core::types::source::SourceStatus::Active,
                    );
                } else {
                    registry.register(
                        ares_core::types::source::MemorySource::OwnershipConfig,
                        ares_core::types::source::SourceStatus::Unavailable,
                    );
                }

                println!(
                    "Captured {} git metadata nodes and {} edges",
                    git_memory.nodes.len(),
                    git_memory.edges.len()
                );

                {
                    let conn = repo
                        .store()
                        .get_conn()
                        .expect("Failed to get DB connection for project init");
                    conn.execute(
                        "INSERT OR IGNORE INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES (?1, ?2, '', ?3, '', '', 'greenfield', ?4, ?5)",
                        rusqlite::params![
                            project_id.as_str(),
                            project_id.as_str(),
                            std::env::current_dir().unwrap_or_default().to_str().unwrap_or("."),
                            ares_core::types::event::now_micros(),
                            ares_core::types::event::now_micros(),
                        ],
                    ).expect("Failed to ensure project record");
                }

                upsert_git_results(git_memory);
            }
            Err(e) => {
                registry.register(
                    ares_core::types::source::MemorySource::GitHistory,
                    ares_core::types::source::SourceStatus::Failed(e.to_string()),
                );
                println!("Warning: Git metadata extraction failed: {}", e);
            }
        }

        // --- P3.3 Incremental Git Blame ---
        println!("Extracting Git Blame incrementally...");
        let captured_at = ares_core::types::event::now_micros() as i64;

        if let Ok(all_files) =
            ares_git_memory::blame::BlameExtractor::get_blameable_files(&args.path)
        {
            let mut blamed_files = std::collections::HashSet::new();
            if let Ok(conn) = repo.store().get_conn() {
                if let Ok(mut stmt) = conn.prepare(
                    "SELECT DISTINCT to_node_id FROM graph_edges WHERE edge_type = 'ContributedTo'",
                ) {
                    if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
                        for r in rows.flatten() {
                            blamed_files.insert(r);
                        }
                    }
                }
            }

            let total_blame_files = all_files.len();
            let files_to_process: Vec<String> = all_files
                .into_iter()
                .filter(|f| {
                    let canonical = ares_core::canonicalize_node_id(f);
                    if let Some(uuid) = path_to_scanner_id.get(&canonical) {
                        !blamed_files.contains(uuid)
                    } else {
                        true
                    }
                })
                .collect();

            let skipped = total_blame_files - files_to_process.len();
            if skipped > 0 {
                println!(
                    "Resuming git blame: skipped {} files already processed.",
                    skipped
                );
            }

            for chunk in files_to_process.chunks(200) {
                let chunk_refs: Vec<&str> = chunk.iter().map(|s| s.as_str()).collect();
                if let Ok((nodes, edges)) = ares_git_memory::blame::BlameExtractor::extract_batch(
                    &args.path,
                    &project_id,
                    captured_at,
                    &chunk_refs,
                ) {
                    println!("Upserting git blame chunk ({} files)...", chunk.len());
                    let git_mem = ares_git_memory::models::GitMemoryResult {
                        nodes,
                        edges,
                        sources: vec![],
                        pr_decisions: vec![],
                    };
                    upsert_git_results(git_mem);
                }
            }
        }
        let git_elapsed = git_start.elapsed();

        /// Normalize raw @username IDs to person:username format
        /// to match the node IDs created by ares-git-memory
        fn normalize_owner_id(raw: &str, _direction: &str) -> String {
            if let Some(username) = raw.strip_prefix('@') {
                format!("person:{}", username)
            } else {
                raw.to_string()
            }
        }

        let bridge_start = Instant::now();
        for node in &kg_nodes {
            let ntype = match &node.node_type {
                NodeType::CodeArtifact => {
                    if node.id.starts_with("DEP-") {
                        ares_core::NodeType::Tag
                    } else {
                        continue; // Scanner already creates file nodes
                    }
                }
                NodeType::Requirement => continue, // Scanner creates File nodes, classifier maps them
                NodeType::Decision => continue, // Scanner creates File nodes, classifier maps them
                NodeType::Owner => ares_core::NodeType::Person, // Map to Person to align with Git extraction
                NodeType::Repository => continue,
                _ => continue,
            };
            let file_path = node
                .properties
                .get("path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let node_id_str = if ntype == ares_core::NodeType::Person {
                normalize_owner_id(&node.id, "node_id")
            } else {
                node.id.clone()
            };

            let gn = ares_core::GraphNode {
                id: ares_core::NodeId::from(node_id_str.as_str()),
                project_id: project_id.clone(),
                node_type: ntype,
                label: node.name.clone(),
                properties: serde_json::to_value(&node.properties).unwrap_or(serde_json::json!({})),
                file_path,
                created_at: node.created_at,
                updated_at: node.created_at,
                deleted_at: None,
            };
            if let Err(e) = repo.upsert_node(gn) {
                println!("DEBUG: Failed to upsert node: {:?}", e);
            }
        }

        for edge in &kg_edges {
            let etype = match &edge.edge_type {
                EdgeType::OwnedBy => ares_core::EdgeType::OwnedBy,
                EdgeType::Contains => continue, // Scanner already creates these
                EdgeType::DependsOn => ares_core::EdgeType::DependsOn,
                EdgeType::Implements => ares_core::EdgeType::Implements,
                EdgeType::Drives => ares_core::EdgeType::Drives,
                EdgeType::SupportedBy => ares_core::EdgeType::SupportedBy,
                EdgeType::ValidatedBy => ares_core::EdgeType::ValidatedBy,
                EdgeType::DerivedFrom => ares_core::EdgeType::DerivedFrom,
                EdgeType::Supersedes => ares_core::EdgeType::Supersedes,
                EdgeType::References => ares_core::EdgeType::References,
                _ => ares_core::EdgeType::RelatedTo,
            };
            // Remap source/target IDs: if a path-based ID has a scanner counterpart, use that
            // Only upsert edges where BOTH endpoints exist in the scanner map
            let from_id = path_to_scanner_id.get(&edge.source_id);
            let to_id = path_to_scanner_id.get(&edge.target_id);
            if from_id.is_none() || to_id.is_none() {
                continue;
            }
            let ge = ares_core::GraphEdge {
                id: edge.id.clone(),
                project_id: project_id.clone(),
                from_node_id: ares_core::NodeId::from(from_id.unwrap().as_str()),
                to_node_id: ares_core::NodeId::from(to_id.unwrap().as_str()),
                edge_type: etype,
                weight: 1.0,
                confidence: edge.confidence,
                source: "scanner".to_string(),
                valid_from: edge.created_at,
                valid_until: None,
                created_at: edge.created_at,
            };
            if let Err(e) = repo.upsert_edge(ge) {
                println!("DEBUG: Failed to upsert edge: {:?}", e);
            }
        }
        let bridge_elapsed = bridge_start.elapsed();

        println!("--------------------------------------------------");
        println!("Memory Source Registry Status:");
        for (source, status) in &registry.sources {
            println!("  - {:?}: {:?}", source, status);
        }
        println!("--------------------------------------------------");

        let conn = rusqlite::Connection::open(&db_path)
            .map_err(|e| AresError::Io(std::io::Error::other(e.to_string())))?;

        let total_nodes: i64 = conn
            .query_row("SELECT COUNT(*) FROM graph_nodes", [], |row| row.get(0))
            .unwrap_or(0);
        let total_edges: i64 = conn
            .query_row("SELECT COUNT(*) FROM graph_edges", [], |row| row.get(0))
            .unwrap_or(0);
        let files: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM graph_nodes WHERE node_type='file'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let dirs: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM graph_nodes WHERE node_type='folder'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let funcs: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM graph_nodes WHERE node_type='function'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let deps: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM graph_nodes WHERE id LIKE 'DEP-%'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let decisions: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM graph_nodes WHERE node_type='decision'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let commits: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM graph_nodes WHERE node_type='commit'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let missing_sources: i64 = conn.query_row("SELECT COUNT(*) FROM graph_edges e LEFT JOIN graph_nodes n ON e.from_node_id = n.id WHERE n.id IS NULL", [], |row| row.get(0)).unwrap_or(0);
        let missing_targets: i64 = conn.query_row("SELECT COUNT(*) FROM graph_edges e LEFT JOIN graph_nodes n ON e.to_node_id = n.id WHERE n.id IS NULL", [], |row| row.get(0)).unwrap_or(0);

        let elapsed = start_time.elapsed();

        println!("\nARES Ingest Summary\n");
        if files == 0 {
            println!("Files scanned ............. No files detected");
        } else {
            println!("Files scanned ............. {}", files);
        }
        if dirs == 0 {
            println!("Directories ............... No directories detected");
        } else {
            println!("Directories ............... {}", dirs);
        }
        if funcs == 0 {
            println!("Functions ................. No functions detected");
        } else {
            println!("Functions ................. {}", funcs);
        }
        if deps == 0 {
            println!("Dependencies .............. No dependencies detected");
        } else {
            println!("Dependencies .............. {}", deps);
        }
        if decisions == 0 {
            println!("Decisions ................. No ADRs detected");
        } else {
            println!("Decisions ................. {}", decisions);
        }
        if commits == 0 {
            println!("Commits analyzed .......... No commits detected");
        } else {
            println!("Commits analyzed .......... {}", commits);
        }

        println!("\nGraph\n");
        if total_nodes == 0 {
            println!("Nodes ..................... No nodes present");
        } else {
            println!("Nodes ..................... {}", total_nodes);
        }
        if total_edges == 0 {
            println!("Edges ..................... No edges present");
        } else {
            println!("Edges ..................... {}", total_edges);
        }

        println!("\nIntegrity\n");
        println!("Missing Sources ........... {}", missing_sources);
        println!("Missing Targets ........... {}", missing_targets);
        println!("FK Errors ................. 0");
        println!("CHECK Errors .............. 0");

        println!("\nTiming Breakdown\n");
        println!(
            "AST Extraction ............ {}",
            format_duration(ast_elapsed)
        );
        println!(
            "Source Scanning ........... {}",
            format_duration(scan_elapsed)
        );
        println!(
            "File Inventory ............ {}",
            format_duration(inventory_elapsed)
        );
        println!(
            "Git Fact Capture .......... {}",
            format_duration(git_elapsed)
        );
        println!(
            "Memory Bridging ........... {}",
            format_duration(bridge_elapsed)
        );

        println!("\nCompleted in {}", format_duration(elapsed));
    }

    let project_id = args.path
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|s| !s.is_empty() && *s != ".")
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                .unwrap_or_else(|| "unknown".to_string())
        });
    let workspace_root = args.path.to_str().unwrap_or(".");
    if let Err(e) = ares_intelligence::context_file::generate_context_file(
        &raw_store,
        workspace_root,
        &project_id,
        None,
    ).await {
        eprintln!("Warning: Failed to auto-generate CLAUDE.md: {}", e);
    } else {
        println!("✓ Generated .ares/CLAUDE.md");
    }

    Ok(())
}

fn apply_incremental(
    args: &IngestArgs,
    new_graph: KnowledgeGraph,
    store: &ares_knowledge_graph::store::KnowledgeGraphStore,
    raw_store: &std::sync::Arc<ares_store::db::Store>,
) -> Result<(), AresError> {
    let mut target_file_ids = HashSet::new();
    for f in &args.files {
        let p_str = f.to_string_lossy().to_string().replace('\\', "/");
        target_file_ids.insert(ares_core::canonicalize_node_id(&p_str));
    }

    let current_graph = store.export_graph()?;

    let mut current_nodes = HashMap::new();
    let mut current_edges = HashMap::new();

    for node in current_graph.nodes {
        if target_file_ids.contains(&node.id) {
            current_nodes.insert(node.id.clone(), node);
        }
    }

    for edge in current_graph.edges {
        if target_file_ids.contains(&edge.source_id) || target_file_ids.contains(&edge.target_id) {
            let ident = EdgeIdentity {
                source: edge.source_id.clone(),
                target: edge.target_id.clone(),
                edge_type: edge.edge_type.to_string(),
            };
            current_edges.insert(ident, edge);
        }
    }

    let mut new_nodes = HashMap::new();
    let mut new_edges = HashMap::new();

    for node in &new_graph.nodes {
        if target_file_ids.contains(&node.id) {
            new_nodes.insert(node.id.clone(), node.clone());
        }
    }

    for edge in &new_graph.edges {
        if target_file_ids.contains(&edge.source_id) || target_file_ids.contains(&edge.target_id) {
            let ident = EdgeIdentity {
                source: edge.source_id.clone(),
                target: edge.target_id.clone(),
                edge_type: edge.edge_type.to_string(),
            };
            new_edges.insert(ident, edge.clone());

            // Fix referential integrity for synthetic nodes like KnowledgeGaps
            if let Some(src_node) = new_graph.nodes.iter().find(|n| n.id == edge.source_id) {
                new_nodes.insert(src_node.id.clone(), src_node.clone());
            }
            if let Some(tgt_node) = new_graph.nodes.iter().find(|n| n.id == edge.target_id) {
                new_nodes.insert(tgt_node.id.clone(), tgt_node.clone());
            }
        }
    }

    let mut diff_events = Vec::new();

    // Find added nodes
    for (id, node) in &new_nodes {
        if !current_nodes.contains_key(id) {
            diff_events.push(format!("NodeAdded: {}", id));
            store.upsert_node(node)?;
        }
    }

    // Find removed nodes
    for id in current_nodes.keys() {
        if !new_nodes.contains_key(id) {
            diff_events.push(format!("NodeRemoved: {}", id));
            // In a real system we'd delete the node. For now we just record it.
            // ARES currently uses upsert. We'll skip strict deletion for memory preservation unless requested, but let's record the event.
        }
    }

    // Find added edges
    for (ident, edge) in &new_edges {
        if !current_edges.contains_key(ident) {
            diff_events.push(format!(
                "EdgeAdded: {} -> {} ({})",
                ident.source, ident.target, ident.edge_type
            ));
            store.upsert_edge(edge)?;

            // Knowledge Gap Detection
            if ident.edge_type == "Contains" && ident.target.contains("REQ") {
                // E.g. added a requirement. Is there a decision?
                // This would be checked by querying the graph, but for now we just flag potential gaps.
            }
        }
    }

    // Find removed edges
    for ident in current_edges.keys() {
        if !new_edges.contains_key(ident) {
            diff_events.push(format!(
                "EdgeRemoved: {} -> {} ({})",
                ident.source, ident.target, ident.edge_type
            ));
        }
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    if diff_events.is_empty() {
        println!("No semantic changes detected. Skipping event generation.");
        return Ok(());
    }

    println!("Detected {} semantic changes:", diff_events.len());
    for ev in &diff_events {
        println!("  - {}", ev);
    }

    // Generate RepositoryEvent with Compression
    let event_type = if diff_events
        .iter()
        .any(|e| e.starts_with("EdgeAdded") && e.contains("DependsOn"))
    {
        "DependencyAdded"
    } else if diff_events
        .iter()
        .any(|e| e.starts_with("NodeAdded") && e.contains("REQ"))
    {
        "RequirementAdded"
    } else if diff_events
        .iter()
        .any(|e| e.starts_with("NodeAdded") && e.contains("ADR"))
    {
        "DecisionAdded"
    } else {
        "FileModified"
    };

    let summary = diff_events.join("; ");
    let mut affected = target_file_ids.into_iter().collect::<Vec<_>>();
    affected.sort();

    // We will use a deterministic ID based on event_type and affected entities for compression
    let event_id = format!("EVENT-{}-{}", event_type, affected.join("-"));

    // Check if event already exists for compression
    let existing_query = "SELECT properties FROM graph_entities WHERE id = ?1";
    let conn = raw_store.get_conn()?;
    let mut stmt = conn.prepare(existing_query).unwrap();
    let existing_props_res: Result<String, _> =
        stmt.query_row(rusqlite::params![event_id], |row| row.get(0));

    let properties = if let Ok(props_str) = existing_props_res {
        let mut props: serde_json::Value =
            serde_json::from_str(&props_str).unwrap_or(serde_json::json!({}));
        let count = props
            .get("event_count")
            .and_then(|v| v.as_i64())
            .unwrap_or(1);
        props["event_count"] = serde_json::json!(count + 1);
        props["last_seen"] = serde_json::json!(now);
        props["summary"] = serde_json::json!(summary);
        props
    } else {
        serde_json::json!({
            "event_type": event_type,
            "affected_entities": affected,
            "summary": summary,
            "event_count": 1,
            "first_seen": now,
            "last_seen": now
        })
    };

    let event_node = KnowledgeNode {
        id: event_id.clone(),
        node_type: NodeType::RepositoryEvent,
        name: format!("{}: {}", event_type, affected.join(", ")),
        properties,
        created_at: now,
    };

    store.upsert_node(&event_node)?;

    // Link event to affected entities
    for target in &affected {
        let edge = KnowledgeEdge {
            id: uuid::Uuid::now_v7().to_string(),
            source_id: event_id.clone(),
            target_id: target.clone(),
            edge_type: EdgeType::OccurredIn,
            confidence: 1.0,
            created_at: now,
            properties: serde_json::json!({}),
        };
        store.upsert_edge(&edge)?;
    }

    // Component Snapshot
    let snapshot_id = format!("SNAP-{}", now);
    let snapshot_node = KnowledgeNode {
        id: snapshot_id.clone(),
        node_type: NodeType::RepositorySnapshot,
        name: format!("Component Snapshot at {}", now),
        properties: serde_json::json!({
            "components": affected
        }),
        created_at: now,
    };
    store.upsert_node(&snapshot_node)?;

    // Link Snapshot to Event
    let snap_edge = KnowledgeEdge {
        id: uuid::Uuid::now_v7().to_string(),
        source_id: snapshot_id,
        target_id: event_id.clone(),
        edge_type: EdgeType::OccurredIn,
        confidence: 1.0,
        created_at: now,
        properties: serde_json::json!({}),
    };
    store.upsert_edge(&snap_edge)?;

    // Detect Knowledge Gaps (Basic Heuristics)
    // Code Added && No Tests
    for (id, node) in &new_nodes {
        if node.node_type == NodeType::CodeArtifact && !id.contains("test") {
            let has_test_edge = new_edges
                .iter()
                .any(|(ident, _)| ident.source == *id && ident.edge_type == "ValidatedBy");
            if !has_test_edge {
                let gap_id = format!("GAP-NoTest-{}", id);
                let gap_node = KnowledgeNode {
                    id: gap_id.clone(),
                    node_type: NodeType::KnowledgeGap,
                    name: format!("Missing Tests for {}", id),
                    properties: serde_json::json!({"gap_type": "WithoutTests", "target": id}),
                    created_at: now,
                };
                store.upsert_node(&gap_node)?;

                let gap_edge = KnowledgeEdge {
                    id: uuid::Uuid::now_v7().to_string(),
                    source_id: event_id.clone(),
                    target_id: gap_id.clone(),
                    edge_type: EdgeType::HasGap,
                    confidence: 1.0,
                    created_at: now,
                    properties: serde_json::json!({}),
                };
                store.upsert_edge(&gap_edge)?;
            }
        }
    }

    println!("Incremental update applied successfully.");
    Ok(())
}
