use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    File,
    Function,
    Module,
    Class,
    Method,
    Struct,
    Enum,
    Trait,
    Interface,
    Service,
    Feature,
    Bug,
    Concept,
    Tag,
    Folder,
    Alternative,
    Assumption,
    Risk,
    ReviewTrigger,
    Person,
    Team,
    Commit,
    Release,
    Branch,
    Capability,
    Requirement,
    RequirementRevision,
    Decision,
    DecisionRevision,
    Evidence,
    Outcome,
    Architecture,
    CodeArtifact,
    Test,
    RuntimeSignal,
    Gap,
    RootCause,
    Resolution,
    Owner,
    Repository,
    Project,
    RepositoryEvent,
    RepositorySnapshot,
    KnowledgeGap,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DomainEventType {
    RequirementCreated,
    RequirementUpdated,
    DecisionCreated,
    DecisionApproved,
    GapDetected,
    ResolutionGenerated,
    ArchitectureDesigned,
    ArchitectureValidated,
    CodeArtifactCommitted,
    TestExecuted,
    TestPassed,
    TestFailed,
    RuntimeSignalDetected,
    RuntimeRegressionDetected,
    OutcomeMeasured,
    OutcomeImproved,
    OutcomeDegraded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub id: String,
    pub event_type: DomainEventType,
    pub entity_id: String,
    pub timestamp: i64,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct ProjectionMetrics {
    pub events_processed: u64,
    pub nodes_created: u64,
    pub edges_created: u64,
    pub duplicate_events_skipped: u64,
    pub projection_failures: u64,
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    pub id: String,
    pub node_type: NodeType,
    pub name: String,
    pub properties: serde_json::Value,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    Imports,
    Defines,
    Calls,
    Extends,
    Implements,
    ImplementedBy,
    Drives,
    DependsOn,
    SupportedBy,
    Supports,
    ValidatedBy,
    ResultsIn,
    OwnedBy,
    Exhibits,
    Causes,
    Resolves,
    References,
    TracesTo,
    ApprovedBy,
    DerivedFrom,
    Supersedes,
    Contains,
    OccurredIn,
    GeneratedFrom,
    HasGap,
}

impl fmt::Display for EdgeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            EdgeType::Imports => "imports",
            EdgeType::Defines => "defines",
            EdgeType::Calls => "calls",
            EdgeType::Extends => "extends",
            EdgeType::Implements => "Implements",
            EdgeType::ImplementedBy => "ImplementedBy",
            EdgeType::Drives => "Drives",
            EdgeType::DependsOn => "DependsOn",
            EdgeType::SupportedBy => "SupportedBy",
            EdgeType::Supports => "Supports",
            EdgeType::ValidatedBy => "ValidatedBy",
            EdgeType::ResultsIn => "ResultsIn",
            EdgeType::OwnedBy => "OwnedBy",
            EdgeType::Exhibits => "Exhibits",
            EdgeType::Causes => "Causes",
            EdgeType::Resolves => "Resolves",
            EdgeType::References => "References",
            EdgeType::TracesTo => "TracesTo",
            EdgeType::ApprovedBy => "ApprovedBy",
            EdgeType::DerivedFrom => "DerivedFrom",
            EdgeType::Supersedes => "Supersedes",
            EdgeType::Contains => "Contains",
            EdgeType::OccurredIn => "OccurredIn",
            EdgeType::GeneratedFrom => "GeneratedFrom",
            EdgeType::HasGap => "HasGap",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub edge_type: EdgeType,
    pub confidence: f32,
    pub created_at: i64,
    pub properties: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: Vec<KnowledgeNode>,
    pub edges: Vec<KnowledgeEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphEvent {
    Node(KnowledgeNode),
    Edge(KnowledgeEdge),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectedEvent {
    pub event_id: String,
    pub projected_at: i64,
}

// Models are now returned directly from queries.rs
