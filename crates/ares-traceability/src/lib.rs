#![allow(clippy::type_complexity)]
#![allow(unused_imports)]
use ares_core::{AresError, GraphEdge, GraphNode};
use petgraph::graph::DiGraph;
use petgraph::visit::Dfs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod test_utils;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TraceTargetType {
    Requirement,
    Decision,
    Architecture,
    Code,
    Test,
    RuntimeMetric,
    Governance,
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceabilityEdge {
    pub source_id: String,
    pub target_id: String,
    pub target_type: TraceTargetType,
    pub relationship: String,
}

#[derive(Debug, Clone)]
pub struct TraceNode {
    pub id: String,
    pub node_type: TraceTargetType,
    pub label: String,
    pub depth: usize,
}

#[derive(Debug, Clone)]
pub struct ImpactReport {
    pub root_id: String,
    pub affected_decisions: Vec<String>,
    pub affected_architecture: Vec<String>,
    pub affected_code: Vec<String>,
    pub affected_requirements: Vec<String>,
    pub total_impact_count: usize,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

pub trait EdgeProvider {
    fn edges(&self) -> Result<Vec<TraceabilityEdge>, AresError>;
}

pub struct TraceabilityGraph {
    providers: Vec<Box<dyn EdgeProvider>>,
}

impl Default for TraceabilityGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceabilityGraph {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn add_provider(&mut self, provider: Box<dyn EdgeProvider>) {
        self.providers.push(provider);
    }

    fn build_graph(
        &self,
    ) -> Result<
        (
            DiGraph<String, String>,
            HashMap<String, petgraph::graph::NodeIndex>,
        ),
        AresError,
    > {
        let mut graph = DiGraph::<String, String>::new();
        let mut nodes = HashMap::new();

        for provider in &self.providers {
            let edges = provider.edges()?;
            for edge in edges {
                let source_idx = *nodes
                    .entry(edge.source_id.clone())
                    .or_insert_with(|| graph.add_node(edge.source_id.clone()));
                let target_idx = *nodes
                    .entry(edge.target_id.clone())
                    .or_insert_with(|| graph.add_node(edge.target_id.clone()));
                graph.add_edge(source_idx, target_idx, edge.relationship);
            }
        }

        Ok((graph, nodes))
    }

    fn guess_type(id: &str) -> TraceTargetType {
        if id.starts_with("REQ-") {
            TraceTargetType::Requirement
        } else if id.starts_with("DEC-") {
            TraceTargetType::Decision
        } else if id.starts_with("ARCH-") {
            TraceTargetType::Architecture
        } else if id.starts_with("CODE-") {
            TraceTargetType::Code
        } else if id.starts_with("TEST-") {
            TraceTargetType::Test
        } else if id.starts_with("METRIC-") || id.starts_with("runtime_metric_") {
            TraceTargetType::RuntimeMetric
        } else if id.starts_with("POLICY-")
            || id.starts_with("GOV-")
            || id.starts_with("TRACE-")
            || id.starts_with("OWNERSHIP-")
        {
            TraceTargetType::Governance
        } else {
            TraceTargetType::Unknown(id.to_string())
        }
    }

    pub fn get_all_nodes(&self) -> Result<Vec<TraceNode>, AresError> {
        let (_, nodes) = self.build_graph()?;
        let mut result = Vec::new();
        for (id, _) in nodes {
            result.push(TraceNode {
                node_type: Self::guess_type(&id),
                id: id.clone(),
                depth: 0,
                label: id,
            });
        }
        Ok(result)
    }

    pub fn get_all_edges(&self) -> Result<Vec<TraceabilityEdge>, AresError> {
        let mut all_edges = Vec::new();
        for provider in &self.providers {
            let mut edges = provider.edges()?;
            all_edges.append(&mut edges);
        }
        Ok(all_edges)
    }

    pub fn find_upstream(&self, id: &str) -> Result<Vec<TraceNode>, AresError> {
        let (graph, nodes) = self.build_graph()?;

        let start_idx = match nodes.get(id) {
            Some(idx) => *idx,
            None => return Ok(vec![]),
        };

        let mut graph_reversed = graph.clone();
        graph_reversed.reverse();

        let mut dfs = Dfs::new(&graph_reversed, start_idx);
        let mut results = Vec::new();

        while let Some(nx) = dfs.next(&graph_reversed) {
            if nx == start_idx {
                continue;
            }
            let node_id = &graph_reversed[nx];
            results.push(TraceNode {
                id: node_id.clone(),
                node_type: Self::guess_type(node_id),
                label: node_id.clone(),
                depth: 1,
            });
        }

        Ok(results)
    }

    pub fn find_downstream(&self, id: &str) -> Result<Vec<TraceNode>, AresError> {
        let (graph, nodes) = self.build_graph()?;

        let start_idx = match nodes.get(id) {
            Some(idx) => *idx,
            None => return Ok(vec![]),
        };

        let mut dfs = Dfs::new(&graph, start_idx);
        let mut results = Vec::new();

        while let Some(nx) = dfs.next(&graph) {
            if nx == start_idx {
                continue;
            }
            let node_id = &graph[nx];
            results.push(TraceNode {
                id: node_id.clone(),
                node_type: Self::guess_type(node_id),
                label: node_id.clone(),
                depth: 1,
            });
        }

        Ok(results)
    }

    pub fn impact_analysis(&self, root_id: &str) -> Result<ImpactReport, AresError> {
        let downstream = self.find_downstream(root_id)?;

        let mut affected_decisions = Vec::new();
        let mut affected_architecture = Vec::new();
        let mut affected_code = Vec::new();
        let mut affected_requirements = Vec::new();

        for node in downstream {
            match node.node_type {
                TraceTargetType::Decision => affected_decisions.push(node.id),
                TraceTargetType::Architecture => affected_architecture.push(node.id),
                TraceTargetType::Code => affected_code.push(node.id),
                TraceTargetType::Requirement => affected_requirements.push(node.id),
                _ => {}
            }
        }

        let total_impact = affected_decisions.len()
            + affected_architecture.len()
            + affected_code.len()
            + affected_requirements.len();

        let risk_level = if total_impact > 20 {
            RiskLevel::Critical
        } else if total_impact > 10 {
            RiskLevel::High
        } else if total_impact > 3 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        Ok(ImpactReport {
            root_id: root_id.to_string(),
            affected_decisions,
            affected_architecture,
            affected_code,
            affected_requirements,
            total_impact_count: total_impact,
            risk_level,
        })
    }
}
