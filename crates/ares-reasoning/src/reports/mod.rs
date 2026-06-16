use crate::bottleneck::BottleneckAnalyzer;
use crate::circular::CircularDependencyAnalyzer;
use crate::dead_code::DeadCodeAnalyzer;
use crate::graph::ReasoningGraph;
use crate::models::{
    Bottleneck, CircularDependency, DeadCodeCandidate, RepositoryHealth, RiskReport,
};
use crate::risk::RiskAnalyzer;
use std::fs;
use std::path::Path;

pub fn generate_reports(graph: &ReasoningGraph, output_dir: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(output_dir)?;

    let dead_code = DeadCodeAnalyzer::analyze(graph);
    let circular = CircularDependencyAnalyzer::analyze(graph);
    let bottlenecks = BottleneckAnalyzer::analyze(graph);
    let risks = RiskAnalyzer::analyze(graph);

    let cycle_penalty = (circular.iter().map(|c| c.severity).sum::<f64>()).min(30.0);
    let total_nodes = graph.nodes.len().max(1) as f64;
    let orphan_ratio = dead_code.len() as f64 / total_nodes;
    let orphan_penalty = (orphan_ratio * 40.0).min(40.0);
    let risk_penalty = (risks.iter().map(|r| r.risk_score).sum::<f64>() / 10.0).min(30.0);

    // Connectivity bonus
    let mut degree_sum = 0;
    for b in &bottlenecks {
        degree_sum += b.degree;
    }
    let avg_degree = if !bottlenecks.is_empty() {
        degree_sum as f64 / bottlenecks.len() as f64
    } else {
        0.0
    };
    let connectivity_bonus = (avg_degree * 0.5).min(20.0);

    let mut health_score =
        100.0 - cycle_penalty - orphan_penalty - risk_penalty + connectivity_bonus;
    health_score = health_score.max(0.0).min(100.0);

    let health = RepositoryHealth {
        score: health_score,
    };

    // write JSON files
    fs::write(
        output_dir.join("dead_code_report.json"),
        serde_json::to_string_pretty(&dead_code)?,
    )?;
    fs::write(
        output_dir.join("circular_dependencies.json"),
        serde_json::to_string_pretty(&circular)?,
    )?;
    fs::write(
        output_dir.join("bottlenecks.json"),
        serde_json::to_string_pretty(&bottlenecks)?,
    )?;
    fs::write(
        output_dir.join("high_risk_files.json"),
        serde_json::to_string_pretty(&risks)?,
    )?;
    fs::write(
        output_dir.join("repository_health.json"),
        serde_json::to_string_pretty(&health)?,
    )?;

    // write dashboard MD
    let md = generate_dashboard_md(&dead_code, &circular, &bottlenecks, &risks, &health);
    fs::write(output_dir.join("REPOSITORY_REASONING_REPORT.md"), &md)?;

    // Also copy to root (which is 2 levels up from artifacts/reasoning)
    if let Some(artifacts) = output_dir.parent() {
        if let Some(root) = artifacts.parent() {
            fs::write(root.join("REPOSITORY_REASONING_REPORT.md"), &md)?;
        }
    }

    Ok(())
}

fn generate_dashboard_md(
    dead_code: &[DeadCodeCandidate],
    circular: &[CircularDependency],
    bottlenecks: &[Bottleneck],
    risks: &[RiskReport],
    health: &RepositoryHealth,
) -> String {
    let mut md = String::new();
    md.push_str("# Repository Reasoning Report\n\n");
    md.push_str(&format!("## Repository Health Score\n\n{:.1}/100\n\n", health.score));
    
    md.push_str("## Dead Code\n");
    md.push_str(&format!("Found {} unused entities.\n\n", dead_code.len()));
    
    md.push_str("## Circular Dependencies\n");
    md.push_str(&format!("Found {} cycles.\n\n", circular.len()));
    
    md.push_str("## Bottlenecks\n");
    md.push_str(&format!("Found {} high-degree nodes.\n\n", bottlenecks.len()));
    
    md.push_str("## High Risk Files\n");
    md.push_str(&format!("Found {} at-risk files.\n\n", risks.len()));
    
    md
}
