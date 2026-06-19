use serde::{Deserialize, Serialize};
use ares_traceability::{TraceabilityGraph, TraceTargetType};
use crate::trace_analysis::TraceAnalysisEngine;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeGapType {
    MissingDecision,
    MissingImplementation,
    MissingTest,
    MissingRuntimeMetric,
    MissingOwner,

    OrphanedDecision,
    OrphanedCode,
    OrphanedTest,
    OrphanedRuntimeMetric,

    UnapprovedRequirement,
    UndocumentedArchitecture,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct KnowledgeGap {
    pub node_id: String,
    pub node_type: String, // "Requirement", "Code", "Decision", etc.
    pub gap_type: KnowledgeGapType,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct GapSummary {
    pub gap_type: KnowledgeGapType,
    pub count: usize,
}

pub struct KnowledgeGapEngine<'a> {
    graph: &'a TraceabilityGraph,
}

impl<'a> KnowledgeGapEngine<'a> {
    pub fn new(graph: &'a TraceabilityGraph) -> Self {
        Self { graph }
    }

    pub fn evaluate_gaps(&self) -> Vec<KnowledgeGap> {
        let resolver = TraceAnalysisEngine::new(&self.graph);
        let mut gaps = Vec::new();

        for node in self.graph.get_all_nodes().unwrap_or_default() {
            match node.node_type {
                TraceTargetType::Requirement => {
                    // Downward gaps are currently computed per-requirement in CoverageEngine.
                    // For a full repository-wide gap analysis, we would check them here.
                    let decisions = resolver.get_downstream(&node.id, TraceTargetType::Decision);
                    if decisions.is_empty() {
                        gaps.push(KnowledgeGap {
                            node_id: node.id.clone(),
                            node_type: "Requirement".to_string(),
                            gap_type: KnowledgeGapType::MissingDecision,
                        });
                    }
                    let code = resolver.get_downstream(&node.id, TraceTargetType::Code);
                    if code.is_empty() {
                        gaps.push(KnowledgeGap {
                            node_id: node.id.clone(),
                            node_type: "Requirement".to_string(),
                            gap_type: KnowledgeGapType::MissingImplementation,
                        });
                    }
                    let tests = resolver.get_downstream(&node.id, TraceTargetType::Test);
                    if tests.is_empty() {
                        gaps.push(KnowledgeGap {
                            node_id: node.id.clone(),
                            node_type: "Requirement".to_string(),
                            gap_type: KnowledgeGapType::MissingTest,
                        });
                    }
                    let metrics = resolver.get_downstream(&node.id, TraceTargetType::RuntimeMetric);
                    if metrics.is_empty() {
                        gaps.push(KnowledgeGap {
                            node_id: node.id.clone(),
                            node_type: "Requirement".to_string(),
                            gap_type: KnowledgeGapType::MissingRuntimeMetric,
                        });
                    }
                }
                TraceTargetType::Decision => {
                    let reqs = resolver.get_upstream(&node.id, TraceTargetType::Requirement);
                    if reqs.is_empty() {
                        gaps.push(KnowledgeGap {
                            node_id: node.id.clone(),
                            node_type: "Decision".to_string(),
                            gap_type: KnowledgeGapType::OrphanedDecision,
                        });
                    }
                }
                TraceTargetType::Code => {
                    let reqs = resolver.get_upstream(&node.id, TraceTargetType::Requirement);
                    if reqs.is_empty() {
                        gaps.push(KnowledgeGap {
                            node_id: node.id.clone(),
                            node_type: "Code".to_string(),
                            gap_type: KnowledgeGapType::OrphanedCode,
                        });
                    }
                }
                TraceTargetType::Test => {
                    let reqs = resolver.get_upstream(&node.id, TraceTargetType::Requirement);
                    if reqs.is_empty() {
                        gaps.push(KnowledgeGap {
                            node_id: node.id.clone(),
                            node_type: "Test".to_string(),
                            gap_type: KnowledgeGapType::OrphanedTest,
                        });
                    }
                }
                TraceTargetType::RuntimeMetric => {
                    let reqs = resolver.get_upstream(&node.id, TraceTargetType::Requirement);
                    if reqs.is_empty() {
                        gaps.push(KnowledgeGap {
                            node_id: node.id.clone(),
                            node_type: "RuntimeMetric".to_string(),
                            gap_type: KnowledgeGapType::OrphanedRuntimeMetric,
                        });
                    }
                }
                TraceTargetType::Architecture => {
                    let reqs = resolver.get_upstream(&node.id, TraceTargetType::Requirement);
                    if reqs.is_empty() {
                        gaps.push(KnowledgeGap {
                            node_id: node.id.clone(),
                            node_type: "Architecture".to_string(),
                            gap_type: KnowledgeGapType::UndocumentedArchitecture,
                        });
                    }
                }
                _ => {}
            }
        }

        gaps
    }

    pub fn generate_summary(&self, gaps: &[KnowledgeGap]) -> Vec<GapSummary> {
        let mut counts = HashMap::new();
        for gap in gaps {
            *counts.entry(gap.gap_type.clone()).or_insert(0) += 1;
        }

        let mut summaries: Vec<GapSummary> = counts
            .into_iter()
            .map(|(gap_type, count)| GapSummary { gap_type, count })
            .collect();

        summaries.sort_by(|a, b| b.count.cmp(&a.count));
        summaries
    }
}
