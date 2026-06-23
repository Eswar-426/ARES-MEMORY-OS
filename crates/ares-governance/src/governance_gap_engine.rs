use ares_core::types::node::NodeType;
use ares_core::AresError;
use ares_core::NodeId;
use ares_core::ProjectId;
use ares_store::repositories::graph::SqliteGraphRepository;
use ares_store::Store;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::ownership_engine::OwnershipEngine;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GovernanceSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GovernanceGapType {
    MissingOwner,
    MissingApprover,
    MissingReviewer,
    MissingCapabilityOwner,
    MissingDecisionAuthority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceGap {
    pub node_id: String,
    pub node_type: NodeType,
    pub gap_type: GovernanceGapType,
    pub severity: GovernanceSeverity,
    pub description: String,
}

pub struct GovernanceGapEngine {
    store: Arc<Store>,
    ownership_engine: OwnershipEngine,
}

impl GovernanceGapEngine {
    pub fn new(store: Arc<Store>) -> Self {
        let ownership_engine = OwnershipEngine::new(store.clone());
        Self {
            store,
            ownership_engine,
        }
    }

    pub fn analyze_node(&self, node_id: &str) -> Result<Vec<GovernanceGap>, AresError> {
        let graph = SqliteGraphRepository::new((*self.store).clone());
        let node_id_obj = NodeId::from(node_id.to_string());
        let node_opt = graph.get_node(&node_id_obj)?;
        let mut gaps = Vec::new();

        if let Some(node) = node_opt {
            let owner = self.ownership_engine.resolve_owner(node_id)?;

            match node.node_type {
                NodeType::Requirement => {
                    if owner.is_none() {
                        gaps.push(GovernanceGap {
                            node_id: node_id.to_string(),
                            node_type: node.node_type.clone(),
                            gap_type: GovernanceGapType::MissingOwner,
                            severity: GovernanceSeverity::Critical,
                            description: "Requirement is missing an authoritative owner."
                                .to_string(),
                        });
                    }
                }
                NodeType::Decision => {
                    // Check for MissingApprover (we will fully implement approval checks in ApprovalGraphEngine,
                    // but we can do a basic check here or defer)
                    // We'll leave the framework here for MissingApprover.
                    // For now, if no owner, it's missing DecisionAuthority.
                    if owner.is_none() {
                        gaps.push(GovernanceGap {
                            node_id: node_id.to_string(),
                            node_type: node.node_type.clone(),
                            gap_type: GovernanceGapType::MissingDecisionAuthority,
                            severity: GovernanceSeverity::High,
                            description: "Decision lacks decision authority (owner).".to_string(),
                        });
                    }
                }
                NodeType::Architecture => {
                    // MissingReviewer checks can be done by checking ReviewedBy edges.
                    // MissingOwner is also relevant here.
                    if owner.is_none() {
                        gaps.push(GovernanceGap {
                            node_id: node_id.to_string(),
                            node_type: node.node_type.clone(),
                            gap_type: GovernanceGapType::MissingOwner,
                            severity: GovernanceSeverity::Medium,
                            description: "Architecture node lacks ownership.".to_string(),
                        });
                    }
                }
                NodeType::Feature => {
                    if owner.is_none() {
                        gaps.push(GovernanceGap {
                            node_id: node_id.to_string(),
                            node_type: node.node_type.clone(),
                            gap_type: GovernanceGapType::MissingCapabilityOwner,
                            severity: GovernanceSeverity::Low,
                            description: "Capability/Feature has no mapped owner.".to_string(),
                        });
                    }
                }
                _ => {
                    // Other types might have gaps too, but prioritizing the main memory tiers.
                }
            }
        }

        Ok(gaps)
    }

    pub fn scan_all(&self, project_id: &ProjectId) -> Result<Vec<GovernanceGap>, AresError> {
        let graph = SqliteGraphRepository::new((*self.store).clone());
        let nodes = graph.get_all_nodes(project_id)?;
        let mut all_gaps = Vec::new();

        for node in nodes {
            let gaps = self.analyze_node(&node.id.to_string())?;
            all_gaps.extend(gaps);
        }

        Ok(all_gaps)
    }
}
