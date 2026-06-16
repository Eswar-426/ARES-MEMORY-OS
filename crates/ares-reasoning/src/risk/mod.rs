use crate::bottleneck::BottleneckAnalyzer;
use crate::graph::ReasoningGraph;
use crate::impact::ImpactAnalyzer;
use crate::models::RiskReport;
use std::collections::HashMap;

pub struct RiskAnalyzer;

impl RiskAnalyzer {
    pub fn analyze(graph: &ReasoningGraph) -> Vec<RiskReport> {
        let bottlenecks = BottleneckAnalyzer::analyze(graph);
        let mut b_map = HashMap::new();
        for b in &bottlenecks {
            b_map.insert(b.node_id.clone(), b.clone());
        }

        let mut file_risks: HashMap<String, (f64, usize, f64, Vec<String>)> = HashMap::new();

        for (id, node) in &graph.nodes {
            if let Some(path) = &node.file_path {
                let impact = ImpactAnalyzer::analyze(graph, id);
                let bottleneck = b_map.get(id);

                let b_score = bottleneck.map(|b| b.risk_score).unwrap_or(0.0);
                let b_deps = bottleneck.map(|b| b.degree).unwrap_or(0);

                let entry = file_risks
                    .entry(path.clone())
                    .or_insert((0.0, 0, 0.0, Vec::new()));
                entry.0 += impact.impact_score;
                entry.1 += b_deps;
                entry.2 += b_score;

                if impact.impact_score > 50.0 {
                    entry.3.push(format!(
                        "Node {} has high impact score ({:.1})",
                        node.label, impact.impact_score
                    ));
                }
                if b_score > 50.0 {
                    entry.3.push(format!(
                        "Node {} is an architectural bottleneck ({:.1})",
                        node.label, b_score
                    ));
                }
            }
        }

        let mut reports = Vec::new();
        for (file, (impact_sum, deps, cent_sum, reasons)) in file_risks {
            let mut deduplicated_reasons = reasons;
            deduplicated_reasons.sort();
            deduplicated_reasons.dedup();

            let score = (impact_sum + deps as f64 + cent_sum) / 3.0;
            if score > 0.0 {
                reports.push(RiskReport {
                    file,
                    risk_score: score.min(100.0), // Cap at 100
                    reasons: deduplicated_reasons,
                });
            }
        }

        reports.sort_by(|a, b| {
            b.risk_score
                .partial_cmp(&a.risk_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        reports
    }
}
