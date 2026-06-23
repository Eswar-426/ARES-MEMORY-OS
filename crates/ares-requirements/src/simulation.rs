use crate::coverage::RequirementCoverageEngine;
use crate::gaps::KnowledgeGapEngine;
use crate::storage::RequirementStore;
use crate::trace_analysis::TraceAnalysisEngine;
use ares_core::{AresError, ProjectId};
use ares_store::Store;
use ares_traceability::{EdgeProvider, TraceTargetType, TraceabilityEdge, TraceabilityGraph};
use std::collections::HashSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposedChange {
    AddNode {
        id: String,
        node_type: TraceTargetType,
    },
    RemoveNode {
        id: String,
    },
    AddEdge {
        source: String,
        target: String,
        relationship: String,
        target_type: TraceTargetType,
    },
    RemoveEdge {
        source: String,
        target: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationReport {
    pub affected_requirements: Vec<String>,
    pub new_gaps: usize,
    pub resolved_gaps: usize,
    pub new_drift: usize,
    pub resolved_drift: usize,
    pub coverage_delta: f32,
    pub blast_radius: usize,
}

struct InMemoryEdgeProvider {
    edges: Vec<TraceabilityEdge>,
}

impl EdgeProvider for InMemoryEdgeProvider {
    fn edges(&self) -> Result<Vec<TraceabilityEdge>, AresError> {
        Ok(self.edges.clone())
    }
}

pub struct RequirementSimulationEngine {
    store: Arc<Store>,
}

impl RequirementSimulationEngine {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    pub fn simulate_change(
        &self,
        project_id: &ProjectId,
        base_graph: &TraceabilityGraph,
        change: ProposedChange,
    ) -> Result<SimulationReport, AresError> {
        let req_store = RequirementStore::new((*self.store).clone());
        let all_reqs = req_store.list(project_id, crate::models::RequirementFilter::default())?;

        // --- BASELINE ---
        let base_trace = TraceAnalysisEngine::new(base_graph);
        let coverage_engine = RequirementCoverageEngine::new();

        let mut base_coverages = Vec::new();
        for req in &all_reqs {
            let cov =
                coverage_engine.evaluate(&req.id, &req.status, req.owner.is_some(), &base_trace);
            base_coverages.push(cov);
        }
        let (base_cov_summary, _) = coverage_engine.generate_summary(&base_coverages);

        let base_gap_engine = KnowledgeGapEngine::new(base_graph);
        let base_gaps = base_gap_engine.evaluate_gaps();

        // --- GRAPH MUTATION ---
        let mut current_edges = base_graph.get_all_edges()?;
        let mut affected_nodes = HashSet::new();

        match &change {
            ProposedChange::AddNode { id, .. } => {
                affected_nodes.insert(id.clone());
            }
            ProposedChange::RemoveNode { id } => {
                affected_nodes.insert(id.clone());
                let upstream_nodes = base_trace.get_upstream_all(id);
                let downstream_nodes = base_trace.get_downstream_all(id);
                for n in upstream_nodes {
                    affected_nodes.insert(n.id);
                }
                for n in downstream_nodes {
                    affected_nodes.insert(n.id);
                }

                current_edges.retain(|e| e.source_id != *id && e.target_id != *id);
            }
            ProposedChange::AddEdge {
                source,
                target,
                relationship,
                target_type,
            } => {
                affected_nodes.insert(source.clone());
                affected_nodes.insert(target.clone());
                current_edges.push(TraceabilityEdge {
                    source_id: source.clone(),
                    target_id: target.clone(),
                    relationship: relationship.clone(),
                    target_type: target_type.clone(),
                });
            }
            ProposedChange::RemoveEdge { source, target } => {
                affected_nodes.insert(source.clone());
                affected_nodes.insert(target.clone());
                current_edges.retain(|e| !(e.source_id == *source && e.target_id == *target));
            }
        }

        let blast_radius = affected_nodes.len();

        let simulated_provider = InMemoryEdgeProvider {
            edges: current_edges,
        };
        let mut simulated_graph = TraceabilityGraph::new();
        simulated_graph.add_provider(Box::new(simulated_provider));

        // --- SIMULATION ---
        let sim_trace = TraceAnalysisEngine::new(&simulated_graph);

        let mut sim_coverages = Vec::new();
        for req in &all_reqs {
            let cov =
                coverage_engine.evaluate(&req.id, &req.status, req.owner.is_some(), &sim_trace);
            sim_coverages.push(cov);
        }
        let (sim_cov_summary, _) = coverage_engine.generate_summary(&sim_coverages);

        let sim_gap_engine = KnowledgeGapEngine::new(&simulated_graph);
        let sim_gaps = sim_gap_engine.evaluate_gaps();

        // --- DELTAS ---
        let coverage_delta = sim_cov_summary.average_coverage - base_cov_summary.average_coverage;
        let new_gaps = sim_gaps.len().saturating_sub(base_gaps.len());
        let resolved_gaps = base_gaps.len().saturating_sub(sim_gaps.len());

        // Drift is computed separately in future phase or based on actual baseline DB.
        let new_drift = 0;
        let resolved_drift = 0;

        let mut affected_requirements = Vec::new();
        for node in affected_nodes {
            if node.starts_with("REQ-") {
                affected_requirements.push(node);
            }
        }

        Ok(SimulationReport {
            affected_requirements,
            new_gaps,
            resolved_gaps,
            new_drift,
            resolved_drift,
            coverage_delta,
            blast_radius,
        })
    }
}
