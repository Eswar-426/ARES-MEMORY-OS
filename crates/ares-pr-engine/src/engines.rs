use ares_core::AresError;
use ares_governance::models::ComplianceViolation;
use ares_knowledge_graph::models::{EdgeType, KnowledgeGraph, NodeType};

use crate::models::{GraphDelta, MemoryImpactReport, MemorySnapshot, MergeReadiness};

pub struct PullRequestEvaluator;

impl PullRequestEvaluator {
    pub fn evaluate(
        base: &MemorySnapshot,
        head: &MemorySnapshot,
    ) -> Result<MergeReadiness, AresError> {
        let graph_delta = GraphDeltaEngine::compute(&base.graph, &head.graph);

        let traceability_links_removed =
            TraceabilityDeltaEngine::compute_removed_links(&base.graph, &head.graph);
        let ownership_changes =
            TraceabilityDeltaEngine::compute_ownership_changes(&base.graph, &head.graph);

        let decisions_affected = DecisionDeltaEngine::compute_affected(&base.graph, &head.graph);
        let requirements_affected =
            DecisionDeltaEngine::compute_requirements_affected(&base.graph, &head.graph);

        let gov_delta = GovernanceDeltaEngine::compute(&base.compliance, &head.compliance);

        let risk_level = ares_governance::risk_engine::RiskClassificationEngine::classify_risk(
            &gov_delta.new_violations_list,
            traceability_links_removed,
            decisions_affected,
            ownership_changes,
        );

        let impact = MemoryImpactReport {
            requirements_affected,
            decisions_affected,
            traceability_links_removed,
            ownership_changes,
            new_compliance_violations: gov_delta.new_violations_list.len(),
            resolved_violations: gov_delta.resolved_violations_list.len(),
            new_violations_list: gov_delta.new_violations_list.clone(),
            resolved_violations_list: gov_delta.resolved_violations_list,
            risk_level,
            baseline_source: "base.json".to_string(), // Can be updated externally
            simulated_coverage_delta: 0.0, // TODO: Hook up RequirementSimulationEngine when dynamically analyzing changes
            simulated_new_drift: 0,
            simulated_new_gaps: 0,
        };

        let mut blocking_violations = Vec::new();
        for res in head.compliance.iter() {
            for v in res.violations.iter() {
                if v.enforcement == ares_governance::models::EnforcementAction::Block {
                    blocking_violations.push(v.clone());
                }
            }
        }

        // Optionally flag traceability removal as blocking if we want strict mode.
        let ready = blocking_violations.is_empty()
            && impact.risk_level != ares_governance::models::MemoryRiskLevel::MemoryCritical;

        let mut warnings = Vec::new();
        if impact.traceability_links_removed > 0 {
            warnings.push(format!(
                "Removed {} traceability links",
                impact.traceability_links_removed
            ));
        }

        Ok(MergeReadiness {
            ready,
            blocking_violations,
            warnings,
            impact,
            graph_delta,
        })
    }
}

pub struct GraphDeltaEngine;
impl GraphDeltaEngine {
    pub fn compute(base: &KnowledgeGraph, head: &KnowledgeGraph) -> GraphDelta {
        use std::collections::HashSet;

        let base_nodes: HashSet<_> = base.nodes.iter().map(|n| n.id.clone()).collect();
        let head_nodes: HashSet<_> = head.nodes.iter().map(|n| n.id.clone()).collect();

        let base_edges: HashSet<_> = base.edges.iter().map(|e| e.id.clone()).collect();
        let head_edges: HashSet<_> = head.edges.iter().map(|e| e.id.clone()).collect();

        let added_nodes = head_nodes.difference(&base_nodes).count();
        let removed_nodes = base_nodes.difference(&head_nodes).count();
        let added_edges = head_edges.difference(&base_edges).count();
        let removed_edges = base_edges.difference(&head_edges).count();

        GraphDelta {
            added_nodes,
            removed_nodes,
            added_edges,
            removed_edges,
        }
    }
}

pub struct TraceabilityDeltaEngine;
impl TraceabilityDeltaEngine {
    pub fn compute_removed_links(base: &KnowledgeGraph, head: &KnowledgeGraph) -> usize {
        let is_traceability = |e: &ares_knowledge_graph::models::KnowledgeEdge| -> bool {
            matches!(
                e.edge_type,
                EdgeType::TracesTo | EdgeType::Implements | EdgeType::ResultsIn
            )
        };

        let base_links: std::collections::HashSet<_> = base
            .edges
            .iter()
            .filter(|e| is_traceability(e))
            .map(|e| e.id.clone())
            .collect();
        let head_links: std::collections::HashSet<_> = head
            .edges
            .iter()
            .filter(|e| is_traceability(e))
            .map(|e| e.id.clone())
            .collect();

        base_links.difference(&head_links).count()
    }

    pub fn compute_ownership_changes(base: &KnowledgeGraph, head: &KnowledgeGraph) -> usize {
        let is_ownership = |e: &ares_knowledge_graph::models::KnowledgeEdge| -> bool {
            matches!(e.edge_type, EdgeType::OwnedBy)
        };

        let base_links: std::collections::HashSet<_> = base
            .edges
            .iter()
            .filter(|e| is_ownership(e))
            .map(|e| e.id.clone())
            .collect();
        let head_links: std::collections::HashSet<_> = head
            .edges
            .iter()
            .filter(|e| is_ownership(e))
            .map(|e| e.id.clone())
            .collect();

        base_links.difference(&head_links).count() + head_links.difference(&base_links).count()
    }
}

pub struct DecisionDeltaEngine;
impl DecisionDeltaEngine {
    pub fn compute_affected(base: &KnowledgeGraph, head: &KnowledgeGraph) -> usize {
        let is_decision = |n: &ares_knowledge_graph::models::KnowledgeNode| -> bool {
            matches!(n.node_type, NodeType::Decision | NodeType::DecisionRevision)
        };

        let base_dec: std::collections::HashSet<_> = base
            .nodes
            .iter()
            .filter(|n| is_decision(n))
            .map(|n| n.id.clone())
            .collect();
        let head_dec: std::collections::HashSet<_> = head
            .nodes
            .iter()
            .filter(|n| is_decision(n))
            .map(|n| n.id.clone())
            .collect();

        base_dec.symmetric_difference(&head_dec).count()
    }

    pub fn compute_requirements_affected(base: &KnowledgeGraph, head: &KnowledgeGraph) -> usize {
        let is_req = |n: &ares_knowledge_graph::models::KnowledgeNode| -> bool {
            matches!(
                n.node_type,
                NodeType::Requirement | NodeType::RequirementRevision
            )
        };

        let base_req: std::collections::HashSet<_> = base
            .nodes
            .iter()
            .filter(|n| is_req(n))
            .map(|n| n.id.clone())
            .collect();
        let head_req: std::collections::HashSet<_> = head
            .nodes
            .iter()
            .filter(|n| is_req(n))
            .map(|n| n.id.clone())
            .collect();

        base_req.symmetric_difference(&head_req).count()
    }
}

pub struct GovernanceDelta {
    pub new_violations_list: Vec<ComplianceViolation>,
    pub resolved_violations_list: Vec<ComplianceViolation>,
}

pub struct GovernanceDeltaEngine;
impl GovernanceDeltaEngine {
    pub fn compute(
        base: &Vec<ares_governance::models::ComplianceResult>,
        head: &Vec<ares_governance::models::ComplianceResult>,
    ) -> GovernanceDelta {
        use std::collections::HashMap;

        // Assume violation identity is policy_name + node_id
        let make_key = |v: &ComplianceViolation| format!("{}:{}", v.policy_name, v.node_id);

        let mut base_map = HashMap::new();
        for res in base {
            for v in &res.violations {
                base_map.insert(make_key(v), v.clone());
            }
        }

        let mut head_map = HashMap::new();
        for res in head {
            for v in &res.violations {
                head_map.insert(make_key(v), v.clone());
            }
        }

        let mut new_violations_list = Vec::new();
        for (k, v) in &head_map {
            if !base_map.contains_key(k) {
                new_violations_list.push(v.clone());
            }
        }

        let mut resolved_violations_list = Vec::new();
        for (k, v) in &base_map {
            if !head_map.contains_key(k) {
                resolved_violations_list.push(v.clone());
            }
        }

        GovernanceDelta {
            new_violations_list,
            resolved_violations_list,
        }
    }
}
