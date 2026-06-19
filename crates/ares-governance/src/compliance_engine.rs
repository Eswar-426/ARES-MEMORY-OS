use crate::models::{
    ComplianceResult, ComplianceViolation, PolicyDefinition, PolicyVersion, ViolationSeverity,
};
use ares_core::{AresError, EdgeDirection, EdgeType, NodeId, ProjectId};
use ares_core::types::node::{GraphNode, GraphEdge};
use chrono::Utc;
use uuid::Uuid;

use ares_store::repositories::graph::SqliteGraphRepository;

pub trait GraphProvider {
    fn get_node(&self, id: &NodeId) -> Result<Option<GraphNode>, AresError>;
    fn get_neighbors(
        &self,
        id: &NodeId,
        direction: EdgeDirection,
        edge_types: &[EdgeType],
    ) -> Result<Vec<GraphNode>, AresError>;
    fn get_all_nodes(&self, project_id: &ProjectId) -> Result<Vec<GraphNode>, AresError>;
}

impl GraphProvider for SqliteGraphRepository {
    fn get_node(&self, id: &NodeId) -> Result<Option<GraphNode>, AresError> {
        self.get_node(id)
    }
    fn get_neighbors(
        &self,
        id: &NodeId,
        direction: EdgeDirection,
        edge_types: &[EdgeType],
    ) -> Result<Vec<GraphNode>, AresError> {
        self.get_neighbors(id, direction, edge_types)
    }
    fn get_all_nodes(&self, project_id: &ProjectId) -> Result<Vec<GraphNode>, AresError> {
        self.get_all_nodes(project_id)
    }
}

pub struct ComplianceEngine<P: GraphProvider> {
    graph_repo: P,
}

impl<P: GraphProvider> ComplianceEngine<P> {
    pub fn new(graph_repo: P) -> Self {
        Self { graph_repo }
    }

    pub fn evaluate_node(
        &self,
        project_id: &ProjectId,
        node_id: &NodeId,
        policies: &[(PolicyDefinition, PolicyVersion)],
        exemptions: &[crate::models::PolicyExemption],
    ) -> Result<Vec<ComplianceResult>, AresError> {
        let node = match self.graph_repo.get_node(node_id)? {
            Some(n) => n,
            None => return Err(AresError::not_found("node", node_id.as_str())),
        };

        let mut results = Vec::new();
        tracing::info!("Evaluating node {} ({}) against {} policies", node.id, node.node_type.as_str(), policies.len());

        for (def, version) in policies {
            if !def.spec.rules.iter().any(|r| {
                r.target
                    .iter()
                    .any(|t| t.eq_ignore_ascii_case(node.node_type.as_str()))
            }) {
                continue; // Policy doesn't apply to this node type
            }

            let mut violations = Vec::new();

            for rule in &def.spec.rules {
                tracing::info!("Checking rule {} from policy {} against {}", rule.name, def.metadata.name, node.node_type.as_str());
                if !rule
                    .target
                    .iter()
                    .any(|t| t.eq_ignore_ascii_case(node.node_type.as_str()))
                {
                    continue;
                }

                let mut add_violation = || {
                    tracing::info!("Adding violation for condition {}", rule.condition);
                    violations.push(ComplianceViolation {
                        id: Uuid::now_v7().to_string(),
                        severity: rule.severity.clone(),
                        policy_name: def.metadata.name.clone(),
                        node_id: node.id.to_string(),
                        reason: rule.condition.clone(),
                        supporting_nodes: vec![],
                        enforcement: rule.enforcement.clone(),
                        category: def.metadata.category.clone(),
                    });
                };

                match rule.condition.as_str() {
                    "missing_owner" => {
                        let has_owner = node.properties.get("owners").map_or(false, |o| {
                            o.as_array().map_or(false, |arr| !arr.is_empty())
                        });
                        if !has_owner {
                            add_violation();
                        }
                    }
                    "missing_evidence" => {
                        let neighbors = self.graph_repo.get_neighbors(
                            &node.id,
                            EdgeDirection::Outgoing,
                            &[EdgeType::References, EdgeType::RelatedTo],
                        )?;
                        let has_evidence = neighbors.iter().any(|n| {
                            n.node_type.as_str() == "evidence" || n.node_type.as_str() == "document"
                        });
                        if !has_evidence {
                            add_violation();
                        }
                    }
                    "missing_approval" => {
                        let has_approval = node.properties.get("approvers").map_or(false, |a| {
                            a.as_array().map_or(false, |arr| !arr.is_empty())
                        });
                        if !has_approval {
                            add_violation();
                        }
                    }
                    "missing_traceability" | "broken_traceability" => {
                        if node.node_type.as_str().eq_ignore_ascii_case("file")
                            || node.node_type.as_str().eq_ignore_ascii_case("concept")
                        {
                            let neighbors = self.graph_repo.get_neighbors(
                                &node.id,
                                EdgeDirection::Incoming,
                                &[EdgeType::Implements, EdgeType::Impacts, EdgeType::RelatedTo],
                            )?;
                            if neighbors.is_empty() {
                                add_violation();
                            }
                        } else if node.node_type.as_str().eq_ignore_ascii_case("requirement")
                            || node.node_type.as_str().eq_ignore_ascii_case("decision")
                        {
                            let neighbors = self.graph_repo.get_neighbors(
                                &node.id,
                                EdgeDirection::Outgoing,
                                &[EdgeType::Implements, EdgeType::Impacts, EdgeType::DependsOn, EdgeType::RelatedTo],
                            )?;
                            if neighbors.is_empty() {
                                add_violation();
                            } else {
                                // Optionally record provenance nodes but no violation
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Filter violations using exemptions
            let mut violations = violations.into_iter().filter(|v| {
                !exemptions.iter().any(|ex| {
                    let matches_rule = ex.target_rules.is_empty() || ex.target_rules.contains(&v.policy_name);
                    let matches_node = ex.target_nodes.is_empty() || ex.target_nodes.contains(&v.node_id);
                    if ex.target_rules.is_empty() && ex.target_nodes.is_empty() {
                        false
                    } else {
                        matches_rule && matches_node
                    }
                })
            }).collect::<Vec<_>>();

            results.push(ComplianceResult {
                id: Uuid::now_v7().to_string(),
                project_id: project_id.to_string(),
                entity_id: node.id.to_string(),
                policy_version_id: version.checksum.clone(),
                compliant: violations.is_empty(),
                score: if violations.is_empty() { 100.0 } else { 0.0 },
                evaluated_at: Utc::now().timestamp(),
                violations,
                category: def.metadata.category.clone(),
            });
        }

        Ok(results)
    }

    pub fn evaluate_project(
        &self,
        project_id: &ProjectId,
        policies: &[(PolicyDefinition, PolicyVersion)],
        exemptions: &[crate::models::PolicyExemption],
    ) -> Result<Vec<ComplianceResult>, AresError> {
        let nodes = self.graph_repo.get_all_nodes(project_id)?;
        let mut all_results = Vec::new();

        for node in nodes {
            let res = self.evaluate_node(project_id, &node.id, policies, exemptions)?;
            all_results.extend(res);
        }

        Ok(all_results)
    }
}
