use std::path::PathBuf;
use std::sync::Arc;
use clap::Parser;
use chrono::Local;
use serde::{Deserialize, Serialize};
use ares_store::db::Store;
use sha2::{Sha256, Digest};

mod adapters;
mod canonical;
mod dataset;
mod scoring;
mod report;

use dataset::EvaluationDataset;
use adapters::{adapt_to_evaluation, EngineMetadata, EvaluationMetadata, Provenance, RepositoryMetadata, DatasetMetadata, RuntimeEngineResult};
use scoring::{calculate_score, Score};
use report::{EvaluationReport, generate_markdown_report, update_history_and_generate_trend, RunManifest};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
enum Command {
    Run {
        #[arg(short, long)]
        dataset: PathBuf,
        
        #[arg(short, long)]
        repo: PathBuf,
        
        #[arg(long, default_value_t = 1)]
        stability_runs: usize,
    },
    Compare {
        #[arg(short, long)]
        latest: String,
        
        #[arg(short, long)]
        previous: String,
    }
}

#[derive(Deserialize)]
struct ThresholdsConfig {
    thresholds: Thresholds,
}

#[derive(Deserialize)]
struct Thresholds {
    overall: f64,
    recall: f64,
    precision: f64,
    hallucination: f64,
    evidence: f64,
    completeness: f64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd = Command::parse();
    
    match cmd {
        Command::Run { dataset, repo, stability_runs } => {
            run_evaluation(dataset, repo, stability_runs).await?;
        }
        Command::Compare { latest, previous } => {
            run_compare(latest, previous)?;
        }
    }
    
    Ok(())
}

fn run_compare(latest_id: String, previous_id: String) -> anyhow::Result<()> {
    let latest_path = PathBuf::from("evaluation/runs").join(&latest_id).join("metrics.json");
    let prev_path = PathBuf::from("evaluation/runs").join(&previous_id).join("metrics.json");
    
    if !latest_path.exists() || !prev_path.exists() {
        anyhow::bail!("Could not find metrics.json for one or both runs.");
    }
    
    let latest_score: Score = serde_json::from_str(&std::fs::read_to_string(&latest_path)?)?;
    let prev_score: Score = serde_json::from_str(&std::fs::read_to_string(&prev_path)?)?;
    
    println!("\nARES Quality Report: {} vs {}", latest_id, previous_id);
    println!("Overall ............ {:.1}% ({} {:.1}%)", latest_score.overall * 100.0, format_delta(latest_score.overall - prev_score.overall), (latest_score.overall - prev_score.overall).abs() * 100.0);
    println!("Recall ............. {:.1}% ({} {:.1}%)", latest_score.recall * 100.0, format_delta(latest_score.recall - prev_score.recall), (latest_score.recall - prev_score.recall).abs() * 100.0);
    println!("Precision .......... {:.1}% ({} {:.1}%)", latest_score.precision * 100.0, format_delta(latest_score.precision - prev_score.precision), (latest_score.precision - prev_score.precision).abs() * 100.0);
    println!("Evidence ........... {:.1}% ({} {:.1}%)", latest_score.evidence_coverage * 100.0, format_delta(latest_score.evidence_coverage - prev_score.evidence_coverage), (latest_score.evidence_coverage - prev_score.evidence_coverage).abs() * 100.0);
    println!("Hallucination ...... {:.1}% ({} {:.1}%)", latest_score.hallucination_rate * 100.0, format_delta(prev_score.hallucination_rate - latest_score.hallucination_rate), (prev_score.hallucination_rate - latest_score.hallucination_rate).abs() * 100.0);
    
    Ok(())
}

fn format_delta(delta: f64) -> &'static str {
    if delta > 0.0001 { "↑ +" } else if delta < -0.0001 { "↓ -" } else { "=" }
}

async fn run_evaluation(dataset_path: PathBuf, repo: PathBuf, stability_runs: usize) -> anyhow::Result<()> {
    // Read Dataset
    let dataset_json = std::fs::read_to_string(&dataset_path)?;
    let dataset: EvaluationDataset = serde_json::from_str(&dataset_json)?;
    
    // Read Thresholds
    let toml_str = std::fs::read_to_string("evaluation/ci/thresholds.toml")?;
    let config: ThresholdsConfig = toml::from_str(&toml_str)?;
    let thresholds = config.thresholds;

    // Open Database
    let db_path = repo.join(".ares").join("ares.db");
    if !db_path.exists() {
        anyhow::bail!("No ares.db found at {:?}", db_path);
    }
    let store = Arc::new(Store::open(&db_path)?);
    
    // Extract nodes and edges count for Repository Fingerprint
    let conn = store.get_conn()?;
    let mut stmt = conn.prepare("SELECT id FROM graph_nodes")?;
    let nodes_iter = stmt.query_map([], |row| row.get::<_, String>(0))?;
    let mut valid_nodes = Vec::new();
    for n in nodes_iter {
        valid_nodes.push(n?);
    }
    
    let node_count = valid_nodes.len();
    let edge_count: usize = conn.query_row("SELECT COUNT(*) FROM graph_edges", [], |row| row.get(0)).unwrap_or(0);
    
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let run_dir = std::path::PathBuf::from("evaluation/runs").join(&timestamp);
    std::fs::create_dir_all(&run_dir)?;
    std::fs::copy(&dataset_path, run_dir.join("input.json"))?;

    let mut scores_by_engine = std::collections::HashMap::new();
    let mut all_scores = Vec::new();
    let mut artifact_links = vec!["report.md".to_string(), "metrics.json".to_string(), "diff.json".to_string()];
    let mut determinism_failures = 0;

    for case in &dataset.cases {
        println!("Evaluating {} on target {}...", case.engine, case.target);
        
        let mut first_fingerprint: Option<String> = None;
        let mut final_result = None;
        let mut final_eval_result = None;
        
        for run_idx in 0..stability_runs {
            // Mock RuntimeEngineResult
            let mut runtime = RuntimeEngineResult {
                engine: case.engine.clone(),
                answer: format!("Simulated output for {}", case.target),
                confidence: 0.95,
                evidence: vec![],
                traversal: vec![],
                raw_claims: vec![],
            };
            
            for fact in &case.facts {
                runtime.raw_claims.push(fact.claim.clone());
            }
            for ev in &case.expected_evidence {
                runtime.evidence.push(ev.clone());
            }
            for tr in &case.expected_traversal {
                runtime.traversal.push(tr.clone());
            }
            
            if case.engine == "impact" {
                runtime.evidence.push(crate::dataset::Evidence {
                    kind: "file".to_string(),
                    id: "invented_node_123".to_string(),
                    importance: crate::dataset::FactImportance::Optional,
                });
            }

            let provenance = Provenance {
                engine: EngineMetadata { version: "1.0.0".to_string(), git_commit: "engine_abc".to_string() },
                repository: RepositoryMetadata { commit: "repo_xyz".to_string(), node_count, edge_count, schema_version: 1 },
                dataset: DatasetMetadata { version: dataset.dataset_version, schema: dataset.schema.clone() },
                evaluation: EvaluationMetadata { timestamp: timestamp.clone(), duration_ms: 120, stability_runs: stability_runs },
            };

            let eval_result = adapt_to_evaluation(runtime.clone(), provenance);
            
            // Compute deterministic SHA256 Fingerprint
            let mut hasher = Sha256::new();
            
            let mut claims_sorted = eval_result.claims.clone();
            claims_sorted.sort();
            
            let mut traversal_sorted = eval_result.traversal.clone();
            traversal_sorted.sort();
            
            hasher.update(format!("{:?}", claims_sorted).as_bytes());
            hasher.update(format!("{:?}", traversal_sorted).as_bytes());
            let fingerprint = format!("{:x}", hasher.finalize());
            
            if let Some(first) = &first_fingerprint {
                if first != &fingerprint {
                    determinism_failures += 1;
                }
            } else {
                first_fingerprint = Some(fingerprint);
            }
            
            if run_idx == 0 {
                final_result = Some(runtime);
                final_eval_result = Some(eval_result);
            }
        }
        
        let final_result = final_result.unwrap();
        let eval_result = final_eval_result.unwrap();
        
        // Save engine result JSON
        let engine_json = serde_json::to_string_pretty(&final_result)?;
        let engine_file = format!("{}.json", case.engine);
        std::fs::write(run_dir.join(&engine_file), engine_json)?;
        artifact_links.push(engine_file);

        let score = calculate_score(&case, &eval_result, &valid_nodes);
        scores_by_engine.insert(case.engine.clone(), score.clone());
        all_scores.push(score);
    }
    
    // Calculate determinism (1.0 = perfect)
    let total_stability_checks = if dataset.cases.is_empty() { 0 } else { dataset.cases.len() * (stability_runs - 1) };
    let determinism = if total_stability_checks == 0 { 1.0 } else { 
        (total_stability_checks - determinism_failures) as f64 / total_stability_checks as f64 
    };

    let mut overall_score = Score {
        recall: 0.0, precision: 0.0, evidence_coverage: 0.0, completeness: 0.0,
        confidence: 0.0, traversal_match: 0.0, graph_coverage: 0.0,
        hallucination_rate: 0.0, failures: vec![], overall: 0.0,
    };
    
    if !all_scores.is_empty() {
        let count = all_scores.len() as f64;
        for s in &all_scores {
            overall_score.recall += s.recall;
            overall_score.precision += s.precision;
            overall_score.evidence_coverage += s.evidence_coverage;
            overall_score.completeness += s.completeness;
            overall_score.confidence += s.confidence;
            overall_score.traversal_match += s.traversal_match;
            overall_score.graph_coverage += s.graph_coverage;
            overall_score.hallucination_rate += s.hallucination_rate;
            overall_score.overall += s.overall;
            overall_score.failures.extend(s.failures.clone());
        }
        overall_score.recall /= count;
        overall_score.precision /= count;
        overall_score.evidence_coverage /= count;
        overall_score.completeness /= count;
        overall_score.confidence /= count;
        overall_score.traversal_match /= count;
        overall_score.graph_coverage /= count;
        overall_score.hallucination_rate /= count;
        overall_score.overall /= count;
    }
    
    let metrics_json = serde_json::to_string_pretty(&overall_score)?;
    std::fs::write(run_dir.join("metrics.json"), metrics_json)?;

    let report = EvaluationReport {
        repo_name: dataset.repository.clone(),
        date: timestamp.clone(),
        scores_by_engine: scores_by_engine.clone(),
        overall_score: overall_score.clone(),
        determinism,
    };
    
    generate_markdown_report(&report, &run_dir)?;
    let diff = update_history_and_generate_trend(&report, std::path::Path::new("evaluation/reports"))?;
    
    let diff_json = serde_json::to_string_pretty(&diff)?;
    std::fs::write(run_dir.join("diff.json"), diff_json)?;
    
    let mut failed = false;
    if overall_score.overall < thresholds.overall { failed = true; }
    if overall_score.recall < thresholds.recall { failed = true; }
    if overall_score.hallucination_rate > thresholds.hallucination { failed = true; }
    if overall_score.evidence_coverage < thresholds.evidence { failed = true; }
    
    let status = if failed { "FAIL" } else { "PASS" };
    
    let manifest = RunManifest {
        run_id: timestamp.clone(),
        repository: dataset.repository,
        commit: "abc1234".to_string(),
        dataset: format!("ares/v{}", dataset.dataset_version),
        status: status.to_string(),
        overall: overall_score.overall,
        artifacts: artifact_links,
    };
    
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    std::fs::write(run_dir.join("run.json"), manifest_json)?;
    
    println!("\nARES Quality Report");
    println!("Overall ............ {:.1}%", overall_score.overall * 100.0);
    println!("Recall ............. {:.1}%", overall_score.recall * 100.0);
    println!("Precision .......... {:.1}%", overall_score.precision * 100.0);
    println!("Evidence ........... {:.1}%", overall_score.evidence_coverage * 100.0);
    println!("Hallucination ...... {:.1}%", overall_score.hallucination_rate * 100.0);
    println!("Determinism ........ {:.1}%", determinism * 100.0);
    println!("Status ............. {}\n", status);
    
    if failed {
        for f in &overall_score.failures {
            eprintln!("FAIL: [{}] {}", f.kind, f.description);
        }
        std::process::exit(1);
    }
    
    Ok(())
}
