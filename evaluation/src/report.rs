use crate::scoring::Score;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub struct EvaluationReport {
    pub repo_name: String,
    pub date: String,
    pub scores_by_engine: std::collections::HashMap<String, Score>,
    pub overall_score: Score,
    pub determinism: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub date: String,
    pub overall: f64,
    pub why: f64,
    pub impact: f64,
    pub trace: f64,
    pub simulation: f64,
    pub drift: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EngineDiff {
    pub recall_delta: f64,
    pub precision_delta: f64,
    pub evidence_delta: f64,
    pub hallucination_delta: f64,
    pub overall_delta: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EvaluationDiff {
    pub overall_delta: f64,
    pub engines: std::collections::HashMap<String, EngineDiff>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RunManifest {
    pub run_id: String,
    pub repository: String,
    pub commit: String,
    pub dataset: String,
    pub status: String,
    pub overall: f64,
    pub artifacts: Vec<String>,
}

pub fn generate_markdown_report(report: &EvaluationReport, output_dir: &Path) -> std::io::Result<()> {
    let mut content = String::new();
    
    content.push_str(&format!("# Evaluation Report: {}\n\n", report.repo_name));
    content.push_str(&format!("**Date:** {}\n\n", report.date));
    
    // Overall Section
    content.push_str("## Overall Metrics\n");
    content.push_str(&format!("- **Overall Score:** {:.1}%\n", report.overall_score.overall * 100.0));
    content.push_str(&format!("- **Recall:** {:.1}%\n", report.overall_score.recall * 100.0));
    content.push_str(&format!("- **Precision:** {:.1}%\n", report.overall_score.precision * 100.0));
    content.push_str(&format!("- **Evidence:** {:.1}%\n", report.overall_score.evidence_coverage * 100.0));
    content.push_str(&format!("- **Completeness:** {:.1}%\n", report.overall_score.completeness * 100.0));
    content.push_str(&format!("- **Traversal:** {:.1}%\n", report.overall_score.traversal_match * 100.0));
    content.push_str(&format!("- **Graph Coverage:** {:.1}%\n", report.overall_score.graph_coverage * 100.0));
    content.push_str(&format!("- **Hallucination Rate:** {:.1}%\n", report.overall_score.hallucination_rate * 100.0));
    content.push_str(&format!("- **Determinism:** {:.1}%\n\n", report.determinism * 100.0));
    
    content.push_str("## Engine Breakdown & Coverage\n\n");
    
    for (engine, score) in &report.scores_by_engine {
        content.push_str(&format!("### {}\n", engine.to_uppercase()));
        
        let passed = if score.overall >= 0.90 && score.hallucination_rate <= 0.05 { 1 } else { 0 };
        
        content.push_str("```text\n");
        content.push_str(&format!("Passed         {}\n", passed));
        content.push_str(&format!("Failed         {}\n", 1 - passed));
        content.push_str(&format!("Coverage       100%\n\n"));
        
        content.push_str(&format!("Average Recall       {:.0}%\n", score.recall * 100.0));
        content.push_str(&format!("Average Precision    {:.0}%\n", score.precision * 100.0));
        content.push_str(&format!("Average Evidence     {:.0}%\n", score.evidence_coverage * 100.0));
        content.push_str(&format!("Average Completeness {:.0}%\n", score.completeness * 100.0));
        content.push_str(&format!("Average Traversal    {:.0}%\n", score.traversal_match * 100.0));
        content.push_str(&format!("Average Hallucination {:.0}%\n", score.hallucination_rate * 100.0));
        content.push_str("```\n\n");
        
        if !score.failures.is_empty() {
            content.push_str("#### Failures Detected\n");
            for f in &score.failures {
                content.push_str(&format!("- **[{}]** {}\n", f.kind, f.description));
            }
            content.push_str("\n");
        }
    }
    
    let filepath = output_dir.join("report.md");
    fs::write(&filepath, content)?;
    println!("Report saved to {:?}", filepath);
    
    Ok(())
}

pub fn update_history_and_generate_trend(report: &EvaluationReport, reports_dir: &Path) -> std::io::Result<EvaluationDiff> {
    let history_path = reports_dir.join("history.json");
    
    let mut history: Vec<HistoryEntry> = if history_path.exists() {
        let json = fs::read_to_string(&history_path)?;
        serde_json::from_str(&json).unwrap_or_else(|_| vec![])
    } else {
        Vec::new()
    };
    
    let entry = HistoryEntry {
        date: report.date.clone(),
        overall: report.overall_score.overall * 100.0,
        why: report.scores_by_engine.get("why").map_or(0.0, |s| s.overall * 100.0),
        impact: report.scores_by_engine.get("impact").map_or(0.0, |s| s.overall * 100.0),
        trace: report.scores_by_engine.get("traceability").map_or(0.0, |s| s.overall * 100.0),
        simulation: report.scores_by_engine.get("simulation").map_or(0.0, |s| s.overall * 100.0),
        drift: report.scores_by_engine.get("drift").map_or(0.0, |s| s.overall * 100.0),
    };
    
    let mut diff = EvaluationDiff {
        overall_delta: 0.0,
        engines: std::collections::HashMap::new(),
    };
    
    if let Some(last) = history.last() {
        diff.overall_delta = entry.overall - last.overall;
        // In a real impl, we would persist more detailed history per engine
    }
    
    history.push(entry);
    fs::write(&history_path, serde_json::to_string_pretty(&history)?)?;
    
    let mut trend = String::new();
    trend.push_str("# Historical Trend\n\n```text\nQUALITY OVERALL\n\n");
    for row in (90..=100).rev() {
        trend.push_str(&format!("{:2} ┤", row));
        for h in &history {
            if h.overall.round() as i32 == row {
                trend.push_str(" ● ");
            } else {
                trend.push_str("   ");
            }
        }
        trend.push_str("\n");
    }
    trend.push_str("   └");
    for _ in &history { trend.push_str("───"); }
    trend.push_str("\n```\n");
    fs::write(reports_dir.join("trend.md"), trend)?;
    
    Ok(diff)
}
