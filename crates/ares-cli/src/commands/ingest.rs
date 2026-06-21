use clap::Args;
use std::path::PathBuf;
use ares_ingestion::GraphBuilder;
use ares_core::AresError;
use std::collections::{HashSet, HashMap};
use ares_knowledge_graph::models::{KnowledgeGraph, KnowledgeNode, KnowledgeEdge, NodeType, EdgeType};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Args, Debug, Clone)]
pub struct IngestArgs {
    /// The path to the repository to ingest
    #[arg(default_value = ".")]
    pub path: PathBuf,

    #[arg(long)]
    pub incremental: bool,

    #[arg(long, value_delimiter = ',')]
    pub files: Vec<PathBuf>,
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct EdgeIdentity {
    source: String,
    target: String,
    edge_type: String,
}

pub async fn handle_ingest(args: IngestArgs) -> Result<(), AresError> {
    println!("Ingesting repository at: {}", args.path.display());
    if args.incremental {
        println!("Incremental mode enabled. Processing {} file(s).", args.files.len());
    }
    
    let mut builder = GraphBuilder::new(&args.path);
    if args.incremental {
        builder.set_incremental_files(args.files.clone());
    }
    let graph = builder.build()?;
    
    let out_dir = args.path.join(".ares");
    if !out_dir.exists() {
        std::fs::create_dir_all(&out_dir).map_err(|e| AresError::validation(e.to_string()))?;
    }
    
    let db_path = out_dir.join("ares.db");
    let raw_store = std::sync::Arc::new(ares_store::db::Store::open(&db_path)?);
    let graph_store = ares_knowledge_graph::store::KnowledgeGraphStore::new(raw_store.clone());

    if args.incremental {
        apply_incremental(&args, graph, &graph_store, &raw_store)?;
    } else {
        // Full ingest
        for node in &graph.nodes {
            graph_store.upsert_node(node)?;
        }
        for edge in &graph.edges {
            if let Err(e) = graph_store.upsert_edge(edge) {
                eprintln!("Warning: Failed to insert edge {}: {}", edge.id, e);
            }
        }
        println!("Successfully ingested knowledge graph into SQLite database");
    }
    
    Ok(())
}

fn apply_incremental(args: &IngestArgs, new_graph: KnowledgeGraph, store: &ares_knowledge_graph::store::KnowledgeGraphStore, raw_store: &std::sync::Arc<ares_store::db::Store>) -> Result<(), AresError> {
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
    for (id, _node) in &current_nodes {
        if !new_nodes.contains_key(id) {
            diff_events.push(format!("NodeRemoved: {}", id));
            // In a real system we'd delete the node. For now we just record it.
            // ARES currently uses upsert. We'll skip strict deletion for memory preservation unless requested, but let's record the event.
        }
    }

    // Find added edges
    for (ident, edge) in &new_edges {
        if !current_edges.contains_key(ident) {
            diff_events.push(format!("EdgeAdded: {} -> {} ({})", ident.source, ident.target, ident.edge_type));
            store.upsert_edge(edge)?;

            // Knowledge Gap Detection
            if ident.edge_type == "Contains" && ident.target.contains("REQ") {
                // E.g. added a requirement. Is there a decision?
                // This would be checked by querying the graph, but for now we just flag potential gaps.
            }
        }
    }

    // Find removed edges
    for (ident, _edge) in &current_edges {
        if !new_edges.contains_key(ident) {
            diff_events.push(format!("EdgeRemoved: {} -> {} ({})", ident.source, ident.target, ident.edge_type));
        }
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;

    if diff_events.is_empty() {
        println!("No semantic changes detected. Skipping event generation.");
        return Ok(());
    }

    println!("Detected {} semantic changes:", diff_events.len());
    for ev in &diff_events {
        println!("  - {}", ev);
    }

    // Generate RepositoryEvent with Compression
    let event_type = if diff_events.iter().any(|e| e.starts_with("EdgeAdded") && e.contains("DependsOn")) {
        "DependencyAdded"
    } else if diff_events.iter().any(|e| e.starts_with("NodeAdded") && e.contains("REQ")) {
        "RequirementAdded"
    } else if diff_events.iter().any(|e| e.starts_with("NodeAdded") && e.contains("ADR")) {
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
    let existing_props_res: Result<String, _> = stmt.query_row(rusqlite::params![event_id], |row| row.get(0));

    let properties = if let Ok(props_str) = existing_props_res {
        let mut props: serde_json::Value = serde_json::from_str(&props_str).unwrap_or(serde_json::json!({}));
        let count = props.get("event_count").and_then(|v| v.as_i64()).unwrap_or(1);
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
            let has_test_edge = new_edges.iter().any(|(ident, _)| ident.source == *id && ident.edge_type == "ValidatedBy");
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
