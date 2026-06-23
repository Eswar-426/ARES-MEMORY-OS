use ares_agent_arena::report::ArenaReport;
use std::fs;

fn main() -> anyhow::Result<()> {
    let workspace_root = std::env::current_dir()?;
    let report_path = workspace_root
        .join("artifacts")
        .join("arena")
        .join("arena_reports.json");

    if !report_path.exists() {
        println!(
            "No reports found at {:?}. Run benchmark_agents first.",
            report_path
        );
        return Ok(());
    }

    let content = fs::read_to_string(&report_path)?;
    let reports: Vec<ArenaReport> = serde_json::from_str(&content)?;

    let mut baseline_score = 0.0;
    let mut context_score = 0.0;
    let mut enhanced_score = 0.0;
    let mut planner_score = 0.0;
    let count = reports.len() as f32;

    for report in &reports {
        if let Some(b) = &report.baseline {
            baseline_score += b.confidence_score;
        }
        if let Some(c) = &report.context {
            context_score += c.confidence_score;
        }
        if let Some(e) = &report.enhanced {
            enhanced_score += e.confidence_score;
        }
        if let Some(p) = &report.planner {
            planner_score += p.confidence_score;
        }
    }

    println!("=== ARES AGENT ARENA COMPARISON ===");
    println!("Tasks Evaluated: {}", count);
    println!("-----------------------------------");
    println!(
        "Baseline Agent Average Score:    {:.2}",
        baseline_score / count
    );
    println!(
        "Context Aware Average Score:     {:.2}",
        context_score / count
    );
    println!(
        "Enhanced Context Average Score:  {:.2}",
        enhanced_score / count
    );
    println!(
        "Planner Agent Average Score:     {:.2}",
        planner_score / count
    );

    Ok(())
}
