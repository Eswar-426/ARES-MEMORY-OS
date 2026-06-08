use crate::services::architectural_analysis::ArchitecturalAnalysisEngine;
use ares_core::{KnowledgeGraph, RiskAssessment};

pub struct RiskEngine {
    arch_engine: ArchitecturalAnalysisEngine,
}

impl RiskEngine {
    pub fn new() -> Self {
        Self {
            arch_engine: ArchitecturalAnalysisEngine::new(),
        }
    }
}

impl Default for RiskEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RiskEngine {
    pub fn assess_risk(&self, kg: &KnowledgeGraph) -> RiskAssessment {
        let arch_report = self.arch_engine.analyze(kg);

        // 1. Dependency risk: Based on bottlenecks and hotspots
        let dependency_risk = if kg.nodes.is_empty() {
            0.0
        } else {
            let total_hotspots =
                arch_report.fan_in_hotspots.len() + arch_report.fan_out_hotspots.len();
            let bottlenecks = arch_report.dependency_bottlenecks.len();
            ((total_hotspots + (bottlenecks * 2)) as f64 / kg.nodes.len() as f64).min(1.0)
        };

        // 2. Volatility risk: Based on unstable modules
        let volatility_risk = if kg.nodes.is_empty() {
            0.0
        } else {
            (arch_report.unstable_modules.len() as f64 / kg.nodes.len() as f64).min(1.0)
        };

        // 3. Architectural debt risk: Cycles and orphans
        let debt_risk = if kg.nodes.is_empty() {
            0.0
        } else {
            let cycle_penalty = arch_report.cycles.len() as f64 * 0.1;
            let orphan_penalty = arch_report.orphan_modules.len() as f64 / kg.nodes.len() as f64;
            (cycle_penalty + orphan_penalty).min(1.0)
        };

        // 4. Knowledge risk: Lack of doc/decisions (we approximate based on node types)
        let decision_count = kg
            .nodes
            .iter()
            .filter(|n| n.node_type == ares_core::NodeType::Decision)
            .count();
        let concept_count = kg
            .nodes
            .iter()
            .filter(|n| n.node_type == ares_core::NodeType::Concept)
            .count();
        let total_docs = decision_count + concept_count;
        let knowledge_risk = if kg.nodes.is_empty() {
            0.0
        } else {
            let ratio = total_docs as f64 / kg.nodes.len() as f64;
            (1.0 - ratio).clamp(0.0, 1.0)
        };

        // 5. Overall Risk
        let overall_risk = (dependency_risk * 0.3)
            + (volatility_risk * 0.2)
            + (debt_risk * 0.3)
            + (knowledge_risk * 0.2);

        RiskAssessment {
            overall_risk,
            dependency_risk,
            volatility_risk,
            architectural_debt_risk: debt_risk,
            knowledge_risk,
        }
    }
}
