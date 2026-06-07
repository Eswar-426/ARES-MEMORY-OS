use crate::id::{NodeId, ProjectId};
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────
// Enumerations
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Project,
    File,
    Function,
    Method,
    Class,
    Struct,
    Enum,
    Trait,
    Interface,
    Module,
    Service,
    Decision,
    Feature,
    Bug,
    Concept,
    Tag,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Project  => "project",
            Self::File     => "file",
            Self::Function => "function",
            Self::Method   => "method",
            Self::Class    => "class",
            Self::Struct   => "struct",
            Self::Enum     => "enum",
            Self::Trait    => "trait",
            Self::Interface => "interface",
            Self::Module   => "module",
            Self::Service  => "service",
            Self::Decision => "decision",
            Self::Feature  => "feature",
            Self::Bug      => "bug",
            Self::Concept  => "concept",
            Self::Tag      => "tag",
        }
    }
}

impl std::str::FromStr for NodeType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "project"  => Ok(Self::Project),
            "file"     => Ok(Self::File),
            "function" => Ok(Self::Function),
            "method"   => Ok(Self::Method),
            "class"    => Ok(Self::Class),
            "struct"   => Ok(Self::Struct),
            "enum"     => Ok(Self::Enum),
            "trait"    => Ok(Self::Trait),
            "interface"=> Ok(Self::Interface),
            "module"   => Ok(Self::Module),
            "service"  => Ok(Self::Service),
            "decision" => Ok(Self::Decision),
            "feature"  => Ok(Self::Feature),
            "bug"      => Ok(Self::Bug),
            "concept"  => Ok(Self::Concept),
            "tag"      => Ok(Self::Tag),
            other      => Err(format!("Unknown node type: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    Imports,
    Defines,
    Calls,
    Extends,
    DependsOn,
    Implements,
    Caused,
    FixedBy,
    Supersedes,
    MotivatedBy,
    Impacts,
    Owns,
    Authored,
    RelatedTo,
    TemporalFollows,
    Contradicts,
    Uses,
}

impl EdgeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Imports         => "imports",
            Self::Defines         => "defines",
            Self::Calls           => "calls",
            Self::Extends         => "extends",
            Self::DependsOn       => "depends_on",
            Self::Implements      => "implements",
            Self::Caused          => "caused",
            Self::FixedBy         => "fixed_by",
            Self::Supersedes      => "supersedes",
            Self::MotivatedBy     => "motivated_by",
            Self::Impacts         => "impacts",
            Self::Owns            => "owns",
            Self::Authored        => "authored",
            Self::RelatedTo       => "related_to",
            Self::TemporalFollows => "temporal_follows",
            Self::Contradicts     => "contradicts",
            Self::Uses            => "uses",
        }
    }
}

impl std::str::FromStr for EdgeType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "imports"          => Ok(Self::Imports),
            "defines"          => Ok(Self::Defines),
            "calls"            => Ok(Self::Calls),
            "extends"          => Ok(Self::Extends),
            "depends_on"       => Ok(Self::DependsOn),
            "implements"       => Ok(Self::Implements),
            "caused"           => Ok(Self::Caused),
            "fixed_by"         => Ok(Self::FixedBy),
            "supersedes"       => Ok(Self::Supersedes),
            "motivated_by"     => Ok(Self::MotivatedBy),
            "impacts"          => Ok(Self::Impacts),
            "owns"             => Ok(Self::Owns),
            "authored"         => Ok(Self::Authored),
            "related_to"       => Ok(Self::RelatedTo),
            "temporal_follows" => Ok(Self::TemporalFollows),
            "contradicts"      => Ok(Self::Contradicts),
            "uses"             => Ok(Self::Uses),
            other              => Err(format!("Unknown edge type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeDirection {
    Outgoing,
    Incoming,
    Both,
}

// ─────────────────────────────────────────────────────────────────
// Graph node and edge structs
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id:         NodeId,
    pub project_id: ProjectId,
    pub node_type:  NodeType,
    pub label:      String,
    /// Flexible JSON properties (e.g., function signature, LOC, complexity)
    pub properties: serde_json::Value,
    pub file_path:  Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id:           String,
    pub project_id:   ProjectId,
    pub from_node_id: NodeId,
    pub to_node_id:   NodeId,
    pub edge_type:    EdgeType,
    pub weight:       f32,
    pub confidence:   f32,
    pub source:       String,
    pub valid_from:   i64,
    pub valid_until:  Option<i64>,
    pub created_at:   i64,
}

// ─────────────────────────────────────────────────────────────────
// Impact analysis output types
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactEntry {
    pub node:       GraphNode,
    pub distance:   u8,
    /// Confidence decays 0.1 per hop: depth 1 = 0.9, depth 2 = 0.8, etc.
    pub confidence: f32,
    pub via_edges:  Vec<EdgeType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactGraph {
    pub target:  GraphNode,
    pub impacts: Vec<ImpactEntry>,
}

/// A detected contradiction between two decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    pub node_id:                 NodeId,
    pub node_label:              String,
    pub conflicting_decision_ids: Vec<String>,
    pub description:             String,
    pub confidence:              f32,
}
