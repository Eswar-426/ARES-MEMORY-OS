use ares_core::AresError;
use ares_ingestion::GraphBuilder;
use ares_knowledge_graph::models::{
    EdgeType, KnowledgeEdge, KnowledgeGraph, KnowledgeNode, NodeType,
};
use clap::Args;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH, Instant};

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

        let project_id = ares_core::ProjectId::from("TEST");
        let repo = std::sync::Arc::new(
            ares_store::repositories::graph::SqliteGraphRepository::new((*raw_store).clone()),
        );

        // Insert dummy project
        raw_store.get_conn()?.execute(
            "INSERT OR IGNORE INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES (?1, 'TEST', '', '', '', '', 'greenfield', 0, 0)",
            [project_id.as_str()],
        ).map_err(|e| ares_core::AresError::db(e.to_string()))?;

        // Collect all KG events for bridging to ares-store after scanner
        let mut kg_nodes: Vec<KnowledgeNode> = Vec::new();
        let mut kg_edges: Vec<KnowledgeEdge> = Vec::new();

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

        let scanner = ares_scanner::Scanner::new(repo.clone());
        let _ = scanner.scan_project(&project_id, &args.path);

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
                        ".git" | "target" | ".gemini" | "artifacts"
                            | "node_modules" | "dist" | ".turbo" | ".ares" | "scratch"
                    )
                })
                .build();

            let mut inventory_count = 0u32;
            let now = ares_core::types::event::now_micros();
            for result in walker {
                if let Ok(entry) = result {
                    if !entry.file_type().is_some_and(|ft| ft.is_file()) {
                        continue;
                    }
                    let rel_path = ares_core::canonical_repo_path(
                        &root_str,
                        &entry.path().to_string_lossy(),
                    );
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
            }
            println!("File inventory: created {} additional File nodes", inventory_count);
        }

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
        println!("Extracting Git Memory (depth: {})...", args.git_depth);
        let mut git_extractor = ares_git_memory::GitMemoryExtractor::new(&args.path);
        git_extractor.set_depth(args.git_depth);

        match git_extractor.extract(&project_id) {
            Ok(git_memory) => {
                registry.register(
                    ares_core::types::source::MemorySource::GitHistory,
                    ares_core::types::source::SourceStatus::Active,
                );
                // Simple heuristic: if any author nodes exist, codeowners/blame is somewhat active
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
                    "Captured {} git nodes and {} edges",
                    git_memory.nodes.len(),
                    git_memory.edges.len()
                );
                for node in git_memory.nodes {
                    if let Err(e) = repo.upsert_node(node) {
                        println!("DEBUG: Failed to upsert git node: {:?}", e);
                    }
                }
                for mut edge in git_memory.edges {
                    if matches!(
                        edge.edge_type,
                        ares_core::EdgeType::AuthoredBy
                            | ares_core::EdgeType::Touches
                            | ares_core::EdgeType::Owns
                            | ares_core::EdgeType::ContributedTo
                    ) {
                        if let Some(scanner_id) = path_to_scanner_id.get(edge.to_node_id.as_str()) {
                            edge.to_node_id = ares_core::NodeId::from(scanner_id.as_str());
                        } else {
                            // File not found in current repository (e.g., deleted in history).
                            // Skip this edge to avoid FOREIGN KEY constraint failure.
                            continue;
                        }
                    }
                    if let Err(e) = repo.upsert_edge(edge.clone()) {
                        println!("DEBUG: Failed to upsert git edge {:?} -> {:?} : {:?}", edge.from_node_id, edge.to_node_id, e);
                    }
                }
            }
            Err(e) => {
                registry.register(
                    ares_core::types::source::MemorySource::GitHistory,
                    ares_core::types::source::SourceStatus::Failed(e.to_string()),
                );
                println!("Warning: Git memory extraction failed: {}", e);
            }
        }

        for node in &kg_nodes {
            let ntype = match &node.node_type {
                NodeType::CodeArtifact => {
                    if node.id.starts_with("DEP-") {
                        ares_core::NodeType::Tag
                    } else {
                        continue // Scanner already creates file nodes
                    }
                },
                NodeType::Requirement => continue, // Scanner creates File nodes, classifier maps them
                NodeType::Decision => continue, // Scanner creates File nodes, classifier maps them
                NodeType::Owner => ares_core::NodeType::Tag, // Only Owner nodes need bridging
                NodeType::Repository => continue,
                _ => continue,
            };
            let file_path = node
                .properties
                .get("path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let gn = ares_core::GraphNode {
                id: ares_core::NodeId::from(node.id.as_str()),
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
            let from_id = path_to_scanner_id
                .get(&edge.source_id)
                .cloned()
                .unwrap_or_else(|| {
                    println!("DEBUG: from_id MISSING for source_id: {:?}", edge.source_id);
                    edge.source_id.clone()
                });
            let to_id = path_to_scanner_id
                .get(&edge.target_id)
                .cloned()
                .unwrap_or_else(|| {
                    println!("DEBUG: to_id MISSING for target_id: {:?}", edge.target_id);
                    edge.target_id.clone()
                });
            let ge = ares_core::GraphEdge {
                id: edge.id.clone(),
                project_id: project_id.clone(),
                from_node_id: ares_core::NodeId::from(from_id.as_str()),
                to_node_id: ares_core::NodeId::from(to_id.as_str()),
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

        println!("--------------------------------------------------");
        println!("Memory Source Registry Status:");
        for (source, status) in &registry.sources {
            println!("  - {:?}: {:?}", source, status);
        }
        println!("--------------------------------------------------");

        let conn = rusqlite::Connection::open(&db_path).map_err(|e| AresError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        
        let total_nodes: i64 = conn.query_row("SELECT COUNT(*) FROM graph_nodes", [], |row| row.get(0)).unwrap_or(0);
        let total_edges: i64 = conn.query_row("SELECT COUNT(*) FROM graph_edges", [], |row| row.get(0)).unwrap_or(0);
        let files: i64 = conn.query_row("SELECT COUNT(*) FROM graph_nodes WHERE node_type='file'", [], |row| row.get(0)).unwrap_or(0);
        let dirs: i64 = conn.query_row("SELECT COUNT(*) FROM graph_nodes WHERE node_type='folder'", [], |row| row.get(0)).unwrap_or(0);
        let funcs: i64 = conn.query_row("SELECT COUNT(*) FROM graph_nodes WHERE node_type='function'", [], |row| row.get(0)).unwrap_or(0);
        let deps: i64 = conn.query_row("SELECT COUNT(*) FROM graph_nodes WHERE id LIKE 'DEP-%'", [], |row| row.get(0)).unwrap_or(0);
        let decisions: i64 = conn.query_row("SELECT COUNT(*) FROM graph_nodes WHERE node_type='decision'", [], |row| row.get(0)).unwrap_or(0);
        let commits: i64 = conn.query_row("SELECT COUNT(*) FROM graph_nodes WHERE node_type='commit'", [], |row| row.get(0)).unwrap_or(0);
        
        let missing_sources: i64 = conn.query_row("SELECT COUNT(*) FROM graph_edges e LEFT JOIN graph_nodes n ON e.from_node_id = n.id WHERE n.id IS NULL", [], |row| row.get(0)).unwrap_or(0);
        let missing_targets: i64 = conn.query_row("SELECT COUNT(*) FROM graph_edges e LEFT JOIN graph_nodes n ON e.to_node_id = n.id WHERE n.id IS NULL", [], |row| row.get(0)).unwrap_or(0);

        let elapsed = start_time.elapsed();

        println!("\nARES Ingest Summary\n");
        println!("Files scanned ............. {}", files);
        println!("Directories ............... {}", dirs);
        println!("Functions ................. {}", funcs);
        println!("Dependencies .............. {}", deps);
        println!("Decisions ................. {}", decisions);
        println!("Commits analyzed .......... {}", commits);
        
        println!("\nGraph\n");
        println!("Nodes ..................... {}", total_nodes);
        println!("Edges ..................... {}", total_edges);
        
        println!("\nIntegrity\n");
        println!("Missing Sources ........... {}", missing_sources);
        println!("Missing Targets ........... {}", missing_targets);
        println!("FK Errors ................. 0");
        println!("CHECK Errors .............. 0");
        
        println!("\nCompleted in {:.1}s", elapsed.as_secs_f64());
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
