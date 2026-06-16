use crate::models::{DomainEvent, DomainEventType, KnowledgeNode, KnowledgeEdge, NodeType, EdgeType};
use crate::projection::{GraphProjector, ProjectionBatch};
use ares_core::AresError;
use serde_json::json;

pub struct ProjectorRegistry {
    pub projectors: Vec<Box<dyn GraphProjector>>,
}

impl ProjectorRegistry {
    pub fn new() -> Self {
        Self {
            projectors: Vec::new(),
        }
    }

    pub fn register(&mut self, projector: Box<dyn GraphProjector>) {
        self.projectors.push(projector);
    }
}

pub struct RequirementProjector;

impl GraphProjector for RequirementProjector {
    fn supports(&self, event_type: &DomainEventType) -> bool {
        matches!(
            event_type,
            DomainEventType::RequirementCreated | DomainEventType::RequirementUpdated
        )
    }

    fn project(&self, event: &DomainEvent) -> Result<ProjectionBatch, AresError> {
        let mut batch = ProjectionBatch::new();

        let node = KnowledgeNode {
            id: event.entity_id.clone(),
            node_type: NodeType::Requirement,
            name: event.payload.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown Requirement").to_string(),
            properties: event.payload.clone(),
            created_at: event.timestamp,
        };

        batch.nodes.push(node);
        Ok(batch)
    }
}

pub struct DecisionProjector;

impl GraphProjector for DecisionProjector {
    fn supports(&self, event_type: &DomainEventType) -> bool {
        matches!(
            event_type,
            DomainEventType::DecisionCreated | DomainEventType::DecisionApproved
        )
    }

    fn project(&self, event: &DomainEvent) -> Result<ProjectionBatch, AresError> {
        let mut batch = ProjectionBatch::new();

        let decision_id = event.entity_id.clone();
        
        let decision_node = KnowledgeNode {
            id: decision_id.clone(),
            node_type: NodeType::Decision,
            name: event.payload.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown Decision").to_string(),
            properties: event.payload.clone(),
            created_at: event.timestamp,
        };
        batch.nodes.push(decision_node);

        // Map Owner if approved_by exists
        if let Some(owner) = event.payload.get("approved_by").and_then(|v| v.as_str()) {
            let owner_id = format!("OWNER-{}", owner.to_uppercase());
            let owner_node = KnowledgeNode {
                id: owner_id.clone(),
                node_type: NodeType::Owner,
                name: owner.to_string(),
                properties: json!({}),
                created_at: event.timestamp,
            };
            batch.nodes.push(owner_node);

            let edge = KnowledgeEdge {
                id: "".to_string(), // Replaced by Engine
                source_id: decision_id.clone(),
                target_id: owner_id,
                edge_type: EdgeType::ApprovedBy,
                confidence: 1.0,
                created_at: event.timestamp,
                properties: json!({}),
            };
            batch.edges.push(edge);
        }

        // Map Requirement if requirement_id exists
        if let Some(requirement_id) = event.payload.get("requirement_id").and_then(|v| v.as_str()) {
            batch.edges.push(KnowledgeEdge {
                id: "".to_string(),
                source_id: requirement_id.to_string(),
                target_id: decision_id.clone(),
                edge_type: EdgeType::ResultsIn, // Requirement -> Decision
                confidence: 1.0,
                created_at: event.timestamp,
                properties: json!({}),
            });
        }

        Ok(batch)
    }
}

pub struct GapProjector;

impl GraphProjector for GapProjector {
    fn supports(&self, event_type: &DomainEventType) -> bool {
        matches!(event_type, DomainEventType::GapDetected)
    }

    fn project(&self, event: &DomainEvent) -> Result<ProjectionBatch, AresError> {
        let mut batch = ProjectionBatch::new();

        let gap_id = event.entity_id.clone();
        
        let gap_node = KnowledgeNode {
            id: gap_id.clone(),
            node_type: NodeType::Gap,
            name: event.payload.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown Gap").to_string(),
            properties: event.payload.clone(),
            created_at: event.timestamp,
        };
        batch.nodes.push(gap_node);

        // Extract RootCause if exists
        if let Some(root_cause) = event.payload.get("root_cause").and_then(|v| v.as_str()) {
            let cause_id = format!("CAUSE-{}", gap_id);
            let cause_node = KnowledgeNode {
                id: cause_id.clone(),
                node_type: NodeType::RootCause,
                name: root_cause.to_string(),
                properties: json!({}),
                created_at: event.timestamp,
            };
            batch.nodes.push(cause_node);

            let edge = KnowledgeEdge {
                id: "".to_string(),
                source_id: gap_id.clone(),
                target_id: cause_id,
                edge_type: EdgeType::Causes,
                confidence: 1.0,
                created_at: event.timestamp,
                properties: json!({}),
            };
            batch.edges.push(edge);
        }

        Ok(batch)
    }
}

pub struct ResolutionProjector;

impl GraphProjector for ResolutionProjector {
    fn supports(&self, event_type: &DomainEventType) -> bool {
        matches!(event_type, DomainEventType::ResolutionGenerated)
    }

    fn project(&self, event: &DomainEvent) -> Result<ProjectionBatch, AresError> {
        let mut batch = ProjectionBatch::new();

        let resolution_id = event.entity_id.clone();
        
        let resolution_node = KnowledgeNode {
            id: resolution_id.clone(),
            node_type: NodeType::Resolution,
            name: event.payload.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown Resolution").to_string(),
            properties: event.payload.clone(),
            created_at: event.timestamp,
        };
        batch.nodes.push(resolution_node);

        // Connect back to GAP if target_gap exists
        if let Some(target_gap) = event.payload.get("target_gap").and_then(|v| v.as_str()) {
            let edge = KnowledgeEdge {
                id: "".to_string(),
                source_id: resolution_id.clone(),
                target_id: target_gap.to_string(),
                edge_type: EdgeType::Resolves,
                confidence: 1.0,
                created_at: event.timestamp,
                properties: json!({}),
            };
            batch.edges.push(edge);
        }

        Ok(batch)
    }
}

pub struct ArchitectureProjector;

impl GraphProjector for ArchitectureProjector {
    fn supports(&self, event_type: &DomainEventType) -> bool {
        matches!(
            event_type,
            DomainEventType::ArchitectureDesigned | DomainEventType::ArchitectureValidated
        )
    }

    fn project(&self, event: &DomainEvent) -> Result<ProjectionBatch, AresError> {
        let mut batch = ProjectionBatch::new();
        let node_id = event.entity_id.clone();
        
        batch.nodes.push(KnowledgeNode {
            id: node_id.clone(),
            node_type: NodeType::Architecture,
            name: event.payload.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown Architecture").to_string(),
            properties: event.payload.clone(),
            created_at: event.timestamp,
        });

        if let Some(decision_id) = event.payload.get("decision_id").and_then(|v| v.as_str()) {
            batch.edges.push(KnowledgeEdge {
                id: "".to_string(),
                source_id: decision_id.to_string(),
                target_id: node_id.clone(),
                edge_type: EdgeType::ResultsIn, // Decision -> Architecture
                confidence: 1.0,
                created_at: event.timestamp,
                properties: json!({}),
            });
        }
        Ok(batch)
    }
}

pub struct CodeArtifactProjector;

impl GraphProjector for CodeArtifactProjector {
    fn supports(&self, event_type: &DomainEventType) -> bool {
        matches!(event_type, DomainEventType::CodeArtifactCommitted)
    }

    fn project(&self, event: &DomainEvent) -> Result<ProjectionBatch, AresError> {
        let mut batch = ProjectionBatch::new();
        let node_id = event.entity_id.clone();
        
        batch.nodes.push(KnowledgeNode {
            id: node_id.clone(),
            node_type: NodeType::CodeArtifact,
            name: event.payload.get("file_path").and_then(|v| v.as_str()).unwrap_or("Unknown File").to_string(),
            properties: event.payload.clone(),
            created_at: event.timestamp,
        });

        if let Some(architecture_id) = event.payload.get("architecture_id").and_then(|v| v.as_str()) {
            batch.edges.push(KnowledgeEdge {
                id: "".to_string(),
                source_id: architecture_id.to_string(),
                target_id: node_id.clone(),
                edge_type: EdgeType::Implements, // Architecture -> CodeArtifact
                confidence: 1.0,
                created_at: event.timestamp,
                properties: json!({}),
            });
        }
        Ok(batch)
    }
}

pub struct TestProjector;

impl GraphProjector for TestProjector {
    fn supports(&self, event_type: &DomainEventType) -> bool {
        matches!(
            event_type,
            DomainEventType::TestExecuted | DomainEventType::TestPassed | DomainEventType::TestFailed
        )
    }

    fn project(&self, event: &DomainEvent) -> Result<ProjectionBatch, AresError> {
        let mut batch = ProjectionBatch::new();
        let node_id = event.entity_id.clone();
        
        batch.nodes.push(KnowledgeNode {
            id: node_id.clone(),
            node_type: NodeType::Test,
            name: event.payload.get("test_name").and_then(|v| v.as_str()).unwrap_or("Unknown Test").to_string(),
            properties: event.payload.clone(),
            created_at: event.timestamp,
        });

        if let Some(code_id) = event.payload.get("code_id").and_then(|v| v.as_str()) {
            batch.edges.push(KnowledgeEdge {
                id: "".to_string(),
                source_id: code_id.to_string(),
                target_id: node_id.clone(),
                edge_type: EdgeType::ValidatedBy, // CodeArtifact -> Test
                confidence: 1.0,
                created_at: event.timestamp,
                properties: json!({}),
            });
        }
        Ok(batch)
    }
}

pub struct RuntimeSignalProjector;

impl GraphProjector for RuntimeSignalProjector {
    fn supports(&self, event_type: &DomainEventType) -> bool {
        matches!(
            event_type,
            DomainEventType::RuntimeSignalDetected | DomainEventType::RuntimeRegressionDetected
        )
    }

    fn project(&self, event: &DomainEvent) -> Result<ProjectionBatch, AresError> {
        let mut batch = ProjectionBatch::new();
        let node_id = event.entity_id.clone();
        
        batch.nodes.push(KnowledgeNode {
            id: node_id.clone(),
            node_type: NodeType::RuntimeSignal,
            name: event.payload.get("metric_name").and_then(|v| v.as_str()).unwrap_or("Unknown Metric").to_string(),
            properties: event.payload.clone(),
            created_at: event.timestamp,
        });

        if let Some(test_id) = event.payload.get("test_id").and_then(|v| v.as_str()) {
            batch.edges.push(KnowledgeEdge {
                id: "".to_string(),
                source_id: test_id.to_string(),
                target_id: node_id.clone(),
                edge_type: EdgeType::Exhibits, // Test -> RuntimeSignal
                confidence: 1.0,
                created_at: event.timestamp,
                properties: json!({}),
            });
        }
        Ok(batch)
    }
}

pub struct OutcomeProjector;

impl GraphProjector for OutcomeProjector {
    fn supports(&self, event_type: &DomainEventType) -> bool {
        matches!(
            event_type,
            DomainEventType::OutcomeMeasured | DomainEventType::OutcomeImproved | DomainEventType::OutcomeDegraded
        )
    }

    fn project(&self, event: &DomainEvent) -> Result<ProjectionBatch, AresError> {
        let mut batch = ProjectionBatch::new();
        let node_id = event.entity_id.clone();
        
        batch.nodes.push(KnowledgeNode {
            id: node_id.clone(),
            node_type: NodeType::Outcome,
            name: event.payload.get("outcome_name").and_then(|v| v.as_str()).unwrap_or("Unknown Outcome").to_string(),
            properties: event.payload.clone(),
            created_at: event.timestamp,
        });

        if let Some(signal_id) = event.payload.get("signal_id").and_then(|v| v.as_str()) {
            batch.edges.push(KnowledgeEdge {
                id: "".to_string(),
                source_id: signal_id.to_string(),
                target_id: node_id.clone(),
                edge_type: EdgeType::Causes, // RuntimeSignal -> Outcome. Meaning: RuntimeSignal Causes Outcome. Thus RuntimeSignal is upstream of Outcome.
                // Wait, if RuntimeSignal causes Outcome, then Outcome is CausedBy RuntimeSignal.
                // In my edge direction, A Causes B means A -> B. So RuntimeSignal -> Outcome. Target is Outcome.
                confidence: 1.0,
                created_at: event.timestamp,
                properties: json!({}),
            });
        }
        Ok(batch)
    }
}

pub struct OwnerProjector;

impl GraphProjector for OwnerProjector {
    fn supports(&self, event_type: &DomainEventType) -> bool {
        // Owners can be extracted from many types of events where ownership is defined or transferred.
        matches!(
            event_type,
            DomainEventType::RequirementCreated | DomainEventType::DecisionApproved | DomainEventType::ArchitectureDesigned
        )
    }

    fn project(&self, event: &DomainEvent) -> Result<ProjectionBatch, AresError> {
        let mut batch = ProjectionBatch::new();

        let mut extract_and_push = |key: &str, edge_type: EdgeType, batch: &mut ProjectionBatch| {
            if let Some(owner_name) = event.payload.get(key).and_then(|v| v.as_str()) {
                let owner_id = format!("OWNER-{}", owner_name.to_uppercase().replace(" ", "-"));
                
                batch.nodes.push(KnowledgeNode {
                    id: owner_id.clone(),
                    node_type: NodeType::Owner,
                    name: owner_name.to_string(),
                    properties: json!({}),
                    created_at: event.timestamp,
                });

                // The domain entity is owned/approved by the Owner
                // A ApprovedBy Owner -> A -> Owner
                batch.edges.push(KnowledgeEdge {
                    id: "".to_string(),
                    source_id: event.entity_id.clone(),
                    target_id: owner_id,
                    edge_type,
                    confidence: 1.0,
                    created_at: event.timestamp,
                    properties: json!({}),
                });
            }
        };

        if matches!(event.event_type, DomainEventType::RequirementCreated | DomainEventType::ArchitectureDesigned) {
            extract_and_push("owner", EdgeType::OwnedBy, &mut batch);
        } else if matches!(event.event_type, DomainEventType::DecisionApproved) {
            extract_and_push("approved_by", EdgeType::ApprovedBy, &mut batch);
        }

        Ok(batch)
    }
}
