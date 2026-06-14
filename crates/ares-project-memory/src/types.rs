//! Core types for the Project Memory Engine.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────
// ProjectSnapshot — the complete memory of a project
// ─────────────────────────────────────────────────────────────────

/// A comprehensive snapshot of a project's state, structure, and history.
/// This is the primary output of the Project Memory Engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    pub project_id: String,
    pub name: String,
    pub description: String,
    pub root_path: String,
    pub architecture: ArchitectureProfile,
    pub languages: Vec<LanguageProfile>,
    pub frameworks: Vec<String>,
    pub dependencies: Vec<DependencyInfo>,
    pub folder_structure: FolderTree,
    pub api_endpoints: Vec<ApiEndpoint>,
    pub decisions: Vec<DecisionSummary>,
    pub features: Vec<FeatureSummary>,
    pub bugs: Vec<BugSummary>,
    pub recent_changes: Vec<ChangeRecord>,
    pub stats: ProjectStats,
    pub created_at: i64,
    pub snapshot_version: u32,
}

// ─────────────────────────────────────────────────────────────────
// Architecture
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureProfile {
    pub style: ArchitectureStyle,
    pub components: Vec<ComponentInfo>,
    pub patterns: Vec<String>,
    pub entry_points: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ArchitectureStyle {
    Monolith,
    Microservices,
    Modular,
    Serverless,
    Hybrid,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub name: String,
    pub path: String,
    pub component_type: String,
    pub description: String,
}

// ─────────────────────────────────────────────────────────────────
// Languages & Dependencies
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageProfile {
    pub language: String,
    pub file_count: u32,
    pub line_count: u64,
    pub percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub name: String,
    pub version: String,
    pub dep_type: DependencyType,
    pub source_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DependencyType {
    Runtime,
    Dev,
    Build,
    Peer,
    Optional,
}

// ─────────────────────────────────────────────────────────────────
// Folder structure
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderTree {
    pub name: String,
    pub children: Vec<FolderTree>,
    pub file_count: u32,
    pub is_dir: bool,
}

impl FolderTree {
    pub fn new_dir(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            children: Vec::new(),
            file_count: 0,
            is_dir: true,
        }
    }

    pub fn new_leaf(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            children: Vec::new(),
            file_count: 0,
            is_dir: false,
        }
    }
}

// ─────────────────────────────────────────────────────────────────
// API endpoints
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub method: String,
    pub path: String,
    pub handler: String,
    pub source_file: String,
}

// ─────────────────────────────────────────────────────────────────
// Memory summaries (decisions, features, bugs)
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub reason: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BugSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub severity: String,
    pub created_at: i64,
}

// ─────────────────────────────────────────────────────────────────
// Change tracking
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRecord {
    pub change_type: ChangeType,
    pub description: String,
    pub files_affected: Vec<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    MemoryCreated,
    MemoryUpdated,
    DecisionMade,
    ScanCompleted,
    FeatureAdded,
    BugFixed,
}

// ─────────────────────────────────────────────────────────────────
// Project statistics
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectStats {
    pub total_files: u32,
    pub total_lines: u64,
    pub total_memories: u64,
    pub total_decisions: u64,
    pub total_graph_nodes: u64,
    pub total_graph_edges: u64,
    pub memory_counts_by_type: HashMap<String, u64>,
}
