use ares_agent_arena::report::ArenaReport;
use serde_json::Value;
use std::fs;

fn main() -> anyhow::Result<()> {
    let workspace_root = std::env::current_dir()?;
    let out_dir = workspace_root.join("artifacts").join("validation");
    if !out_dir.exists() {
        fs::create_dir_all(&out_dir)?;
    }

    let mut summary = String::new();
    summary.push_str("# WEEK 29 CONNECTIVITY REPORT\n\n");

    // Graph Health
    summary.push_str("## Graph Health\n");
    let graph_stats_path = out_dir.join("graph_stats.json");
    if graph_stats_path.exists() {
        if let Ok(content) = fs::read_to_string(&graph_stats_path) {
            if let Ok(stats) = serde_json::from_str::<Value>(&content) {
                summary.push_str(&format!("- **Total Nodes**: {}\n", stats["total_nodes"]));
                summary.push_str(&format!("- **Total Edges**: {}\n", stats["total_edges"]));
                summary.push_str(&format!(
                    "- **Largest Connected Component**: {}\n",
                    stats["largest_connected_component"]
                ));
                summary.push_str(&format!(
                    "- **Density**: {:.4}\n",
                    stats["density"].as_f64().unwrap_or(0.0)
                ));
            }
        }
    } else {
        summary.push_str(
            "*Run `cargo run -p ares-store --bin graph_stats` to populate graph health.*\n",
        );
    }
    summary.push('\n');

    // Graph Evolution
    summary.push_str("## Graph Evolution\n");
    summary.push_str("### Baseline (Week 28)\n");
    summary.push_str("- **Nodes**: 9627\n");
    summary.push_str("- **Edges**: 7598\n");
    summary.push_str("- **Largest Component**: 175\n\n");

    summary.push_str("### Current (Week 29)\n");
    if graph_stats_path.exists() {
        if let Ok(content) = fs::read_to_string(&graph_stats_path) {
            if let Ok(stats) = serde_json::from_str::<Value>(&content) {
                let curr_nodes = stats["total_nodes"].as_u64().unwrap_or(0);
                let curr_edges = stats["total_edges"].as_u64().unwrap_or(0);
                let curr_lcc = stats["largest_connected_component"].as_u64().unwrap_or(0);

                summary.push_str(&format!("- **Nodes**: {}\n", curr_nodes));
                summary.push_str(&format!("- **Edges**: {}\n", curr_edges));
                summary.push_str(&format!("- **Largest Component**: {}\n\n", curr_lcc));

                summary.push_str("### Growth\n");
                let growth = if 175 > 0 {
                    ((curr_lcc as f64 - 175.0) / 175.0) * 100.0
                } else {
                    0.0
                };
                summary.push_str(&format!("+**{:.0}%** (Largest Component)\n\n", growth));
            }
        }
    } else {
        summary.push_str(
            "*Run `cargo run -p ares-store --bin graph_stats` to populate current stats.*\n\n",
        );
    }

    // Call Graph Metrics
    summary.push_str("## Call Graph Metrics\n");
    let call_graph_stats_path = out_dir.join("call_graph_metrics.json");
    if call_graph_stats_path.exists() {
        if let Ok(content) = fs::read_to_string(&call_graph_stats_path) {
            if let Ok(stats) = serde_json::from_str::<Value>(&content) {
                summary.push_str(&format!("- **Call Edges**: {}\n", stats["call_edges"]));
                summary.push_str(&format!(
                    "- **Dependency Edges**: {}\n",
                    stats["dependency_edges"]
                ));
                summary.push_str(&format!(
                    "- **Implementation Edges**: {}\n",
                    stats["implementation_edges"]
                ));
                summary.push_str(&format!(
                    "- **Resolved Symbols**: {}\n",
                    stats["resolved_symbols"]
                ));
                summary.push_str(&format!(
                    "- **Unresolved Symbols**: {}\n",
                    stats["unresolved_symbols"]
                ));

                let res = stats["resolved_symbols"].as_f64().unwrap_or(0.0);
                let unres = stats["unresolved_symbols"].as_f64().unwrap_or(0.0);
                let total = res + unres;
                if total > 0.0 {
                    summary.push_str(&format!(
                        "- **Resolution Rate**: {:.2}%\n",
                        (res / total) * 100.0
                    ));
                }
            }
        }
    } else {
        summary.push_str(
            "*Run `cargo run -p ares-store --bin graph_stats` to populate call graph health.*\n",
        );
    }
    summary.push('\n');

    // Scanner Performance
    summary.push_str("## Scanner Performance\n");
    let scanner_report_path = out_dir.join("scanner_report.json");
    if scanner_report_path.exists() {
        if let Ok(content) = fs::read_to_string(&scanner_report_path) {
            if let Ok(stats) = serde_json::from_str::<Value>(&content) {
                summary.push_str(&format!(
                    "- **Files Scanned**: {}\n",
                    stats["files_scanned"]
                ));
                summary.push_str(&format!("- **Parsed Files**: {}\n", stats["parsed_files"]));
                summary.push_str(&format!(
                    "- **Extraction Success**: {:.2}%\n",
                    stats["extraction_success_rate"].as_f64().unwrap_or(0.0) * 100.0
                ));
                summary.push_str(&format!(
                    "- **Symbols Extracted**: {}\n",
                    stats["symbols_extracted"]
                ));
            }
        }
    } else {
        summary.push_str(
            "*Run `cargo run -p ares-scanner --bin scan_self` to populate scanner performance.*\n",
        );
    }
    summary.push('\n');

    // Arena Results
    summary.push_str("## Arena Results\n");
    let report_path = workspace_root
        .join("artifacts")
        .join("arena")
        .join("arena_reports.json");
    if report_path.exists() {
        if let Ok(content) = fs::read_to_string(&report_path) {
            if let Ok(reports) = serde_json::from_str::<Vec<ArenaReport>>(&content) {
                let mut baseline_score = 0.0;
                let mut baseline_cov = 0.0;
                let mut context_score = 0.0;
                let mut context_cov = 0.0;
                let mut enhanced_score = 0.0;
                let mut enhanced_cov = 0.0;
                let mut planner_score = 0.0;
                let mut planner_cov = 0.0;
                let count = reports.len() as f32;

                for report in &reports {
                    if let Some(b) = &report.baseline {
                        baseline_score += b.overall_score;
                        baseline_cov += b.graph_coverage;
                    }
                    if let Some(c) = &report.context {
                        context_score += c.overall_score;
                        context_cov += c.graph_coverage;
                    }
                    if let Some(e) = &report.enhanced {
                        enhanced_score += e.overall_score;
                        enhanced_cov += e.graph_coverage;
                    }
                    if let Some(p) = &report.planner {
                        planner_score += p.overall_score;
                        planner_cov += p.graph_coverage;
                    }
                }

                summary.push_str("| Agent Type | Overall Score | Graph Coverage |\n");
                summary.push_str("|------------|---------------|----------------|\n");
                summary.push_str(&format!(
                    "| Baseline | {:.2} | {:.2}% |\n",
                    baseline_score / count,
                    (baseline_cov / count) * 100.0
                ));
                summary.push_str(&format!(
                    "| Context Aware | {:.2} | {:.2}% |\n",
                    context_score / count,
                    (context_cov / count) * 100.0
                ));
                summary.push_str(&format!(
                    "| Enhanced Context | {:.2} | {:.2}% |\n",
                    enhanced_score / count,
                    (enhanced_cov / count) * 100.0
                ));
                summary.push_str(&format!(
                    "| Planner | {:.2} | {:.2}% |\n",
                    planner_score / count,
                    (planner_cov / count) * 100.0
                ));
            }
        }
    } else {
        summary.push_str("*Run `cargo run -p ares-agent-arena --bin benchmark_agents` to populate arena results.*\n");
    }

    let summary_path = workspace_root.join("WEEK_29_CONNECTIVITY_REPORT.md");
    fs::write(&summary_path, summary)?;
    println!("Dashboard summary written to {:?}", summary_path);

    Ok(())
}
