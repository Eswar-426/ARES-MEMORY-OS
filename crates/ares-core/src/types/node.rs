use crate::id::{NodeId, ProjectId};
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────
// Enumerations
// ─────────────────────────────────────────────────────────────────

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Project,
    Module,
    File,
    Function,
    Method,
    Class,
    Struct,
    Enum,
    Trait,
    Interface,
    Service,
    Decision,
    Feature,
    Bug,
    Concept,
    Tag,
    Folder,
    Alternative,
    Assumption,
    Risk,
    Requirement,
    // P3.2 Memory Capture: fact-level node types
    Person,
    Team,
    Commit,
    Release,
    Branch,
    // P3.4 Reasoning Node Types
    Architecture,
    Test,
    RuntimeSignal,
    Outcome,
    // P3.5 Evolution Engine
    EvolutionEvent,
    Evidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolSignature {
    pub name: String,
    pub file_path: Option<String>,
    pub module_path: Option<String>,
    pub symbol_type: NodeType,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::File => "file",
            Self::Function => "function",
            Self::Method => "method",
            Self::Class => "class",
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Trait => "trait",
            Self::Interface => "interface",
            Self::Module => "module",
            Self::Service => "service",
            Self::Decision => "decision",
            Self::Feature => "feature",
            Self::Bug => "bug",
            Self::Concept => "concept",
            Self::Tag => "tag",
            Self::Folder => "folder",
            Self::Alternative => "alternative",
            Self::Assumption => "assumption",
            Self::Risk => "risk",
            Self::Requirement => "requirement",
            Self::Person => "person",
            Self::Team => "team",
            Self::Commit => "commit",
            Self::Release => "release",
            Self::Branch => "branch",
            Self::Architecture => "architecture",
            Self::Test => "test",
            Self::RuntimeSignal => "runtime_signal",
            Self::Outcome => "outcome",
            Self::EvolutionEvent => "evolution_event",
            Self::Evidence => "evidence",
        }
    }
}

impl std::str::FromStr for NodeType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "project" => Ok(Self::Project),
            "file" => Ok(Self::File),
            "function" => Ok(Self::Function),
            "method" => Ok(Self::Method),
            "class" => Ok(Self::Class),
            "struct" => Ok(Self::Struct),
            "enum" => Ok(Self::Enum),
            "trait" => Ok(Self::Trait),
            "interface" => Ok(Self::Interface),
            "module" => Ok(Self::Module),
            "service" => Ok(Self::Service),
            "decision" => Ok(Self::Decision),
            "feature" => Ok(Self::Feature),
            "bug" => Ok(Self::Bug),
            "concept" => Ok(Self::Concept),
            "tag" => Ok(Self::Tag),
            "folder" => Ok(Self::Folder),
            "alternative" => Ok(Self::Alternative),
            "assumption" => Ok(Self::Assumption),
            "risk" => Ok(Self::Risk),
            "requirement" => Ok(Self::Requirement),
            "person" => Ok(Self::Person),
            "team" => Ok(Self::Team),
            "commit" => Ok(Self::Commit),
            "release" => Ok(Self::Release),
            "branch" => Ok(Self::Branch),
            "architecture" => Ok(Self::Architecture),
            "test" => Ok(Self::Test),
            "runtime_signal" => Ok(Self::RuntimeSignal),
            "outcome" => Ok(Self::Outcome),
            "evolution_event" => Ok(Self::EvolutionEvent),
            "evidence" => Ok(Self::Evidence),
            other => Err(format!("Unknown node type: {other}")),
        }
    }
}

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
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
    DerivedFrom,
    Contains,
    ContainedIn,
    Invokes,
    Constructs,
    References,
    ResolvedTo,
    UsesModule,
    UsesTrait,
    Constrains,
    HasRisk,
    HasAssumption,
    Drives,
    Satisfies,
    OwnedBy,
    SupportedBy,
    ValidatedBy,
    // P3.2 Memory Capture: contribution and evolution edges
    ContributedTo,
    Maintains,
    Touches,
    AuthoredBy,
    ReleasedIn,
    // P3.5 Evolution Engine
    Evolves,
    Drifts,
    Supports,
    Observes,
    // P5 Governance Intelligence
    MemberOf,
    ApprovedBy,
    ReviewedBy,
}

impl EdgeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Imports => "imports",
            Self::Defines => "defines",
            Self::Calls => "calls",
            Self::Extends => "extends",
            Self::DependsOn => "depends_on",
            Self::Implements => "implements",
            Self::Caused => "caused",
            Self::FixedBy => "fixed_by",
            Self::Supersedes => "supersedes",
            Self::MotivatedBy => "motivated_by",
            Self::Impacts => "impacts",
            Self::Owns => "owns",
            Self::Authored => "authored",
            Self::RelatedTo => "related_to",
            Self::TemporalFollows => "temporal_follows",
            Self::Contradicts => "contradicts",
            Self::Uses => "uses",
            Self::DerivedFrom => "derived_from",
            Self::Contains => "contains",
            Self::ContainedIn => "contained_in",
            Self::Invokes => "invokes",
            Self::Constructs => "constructs",
            Self::References => "references",
            Self::ResolvedTo => "resolved_to",
            Self::UsesModule => "uses_module",
            Self::UsesTrait => "uses_trait",
            Self::Constrains => "constrains",
            Self::HasRisk => "has_risk",
            Self::HasAssumption => "has_assumption",
            Self::Drives => "drives",
            Self::Satisfies => "satisfies",
            Self::OwnedBy => "owned_by",
            Self::SupportedBy => "supported_by",
            Self::ValidatedBy => "validated_by",
            Self::ContributedTo => "contributed_to",
            Self::Maintains => "maintains",
            Self::Touches => "touches",
            Self::AuthoredBy => "authored_by",
            Self::ReleasedIn => "released_in",
            Self::Evolves => "evolves",
            Self::Drifts => "drifts",
            Self::Supports => "supports",
            Self::Observes => "observes",
            Self::MemberOf => "member_of",
            Self::ApprovedBy => "approved_by",
            Self::ReviewedBy => "reviewed_by",
        }
    }
}

impl std::str::FromStr for EdgeType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "imports" => Ok(Self::Imports),
            "defines" => Ok(Self::Defines),
            "calls" => Ok(Self::Calls),
            "extends" => Ok(Self::Extends),
            "depends_on" => Ok(Self::DependsOn),
            "implements" => Ok(Self::Implements),
            "caused" => Ok(Self::Caused),
            "fixed_by" => Ok(Self::FixedBy),
            "supersedes" => Ok(Self::Supersedes),
            "motivated_by" => Ok(Self::MotivatedBy),
            "impacts" => Ok(Self::Impacts),
            "owns" => Ok(Self::Owns),
            "authored" => Ok(Self::Authored),
            "related_to" => Ok(Self::RelatedTo),
            "temporal_follows" => Ok(Self::TemporalFollows),
            "contradicts" => Ok(Self::Contradicts),
            "uses" => Ok(Self::Uses),
            "derived_from" => Ok(Self::DerivedFrom),
            "contains" => Ok(Self::Contains),
            "contained_in" => Ok(Self::ContainedIn),
            "invokes" => Ok(Self::Invokes),
            "constructs" => Ok(Self::Constructs),
            "references" => Ok(Self::References),
            "resolved_to" => Ok(Self::ResolvedTo),
            "uses_module" => Ok(Self::UsesModule),
            "uses_trait" => Ok(Self::UsesTrait),
            "constrains" => Ok(Self::Constrains),
            "has_risk" => Ok(Self::HasRisk),
            "has_assumption" => Ok(Self::HasAssumption),
            "drives" => Ok(Self::Drives),
            "satisfies" => Ok(Self::Satisfies),
            "owned_by" => Ok(Self::OwnedBy),
            "supported_by" => Ok(Self::SupportedBy),
            "validated_by" => Ok(Self::ValidatedBy),
            "contributed_to" => Ok(Self::ContributedTo),
            "maintains" => Ok(Self::Maintains),
            "touches" => Ok(Self::Touches),
            "authored_by" => Ok(Self::AuthoredBy),
            "released_in" => Ok(Self::ReleasedIn),
            "evolves" => Ok(Self::Evolves),
            "drifts" => Ok(Self::Drifts),
            "supports" => Ok(Self::Supports),
            "observes" => Ok(Self::Observes),
            "member_of" => Ok(Self::MemberOf),
            "approved_by" => Ok(Self::ApprovedBy),
            "reviewed_by" => Ok(Self::ReviewedBy),
            other => Err(format!("Unknown edge type: {other}")),
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

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: NodeId,
    pub project_id: ProjectId,
    pub node_type: NodeType,
    pub label: String,
    /// Flexible JSON properties (e.g., function signature, LOC, complexity)
    pub properties: serde_json::Value,
    pub file_path: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub project_id: ProjectId,
    pub from_node_id: NodeId,
    pub to_node_id: NodeId,
    pub edge_type: EdgeType,
    pub weight: f32,
    pub confidence: f32,
    pub source: String,
    pub valid_from: i64,
    pub valid_until: Option<i64>,
    pub created_at: i64,
}

// ─────────────────────────────────────────────────────────────────
// Impact analysis output types
// ─────────────────────────────────────────────────────────────────

#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactEntry {
    pub node: GraphNode,
    pub distance: u8,
    /// Confidence decays 0.1 per hop: depth 1 = 0.9, depth 2 = 0.8, etc.
    pub confidence: f32,
    pub via_edges: Vec<EdgeType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ImpactGraph {
    pub target: GraphNode,
    pub impacts: Vec<ImpactEntry>,
}

/// A detected contradiction between two nodes (e.g., decisions, memories).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    pub source_id: NodeId,
    pub target_id: NodeId,
    pub reason: String,
    pub confidence: f32,
}
