use ares_core::AresError;
use ares_reasoning::{BreakageEngine, ImpactEngine, WhyEngine, GapEngine};
use ares_store::Store;
use ares_core::ProjectId;

pub async fn handle_why(_target_type: &str, target_id: &str) -> Result<(), AresError> {
    let store = Store::open(std::path::Path::new(".ares/ares.db"))?;
    let engine = WhyEngine::new(store);

    let report = engine.explain(target_id)?;

    println!("═══════════════════════════════════════════════");
    println!("  ARES WHY — Lineage Report");
    println!("═══════════════════════════════════════════════");
    println!("  Target: {}", report.target_id);
    println!("  Status: {}", report.status);
    println!("───────────────────────────────────────────────");

    if !report.requirements.is_empty() {
        println!("  Requirements:");
        for r in &report.requirements {
            println!("    • {}", r);
        }
    } else {
        println!("  Requirements: Unknown");
    }

    if !report.decisions.is_empty() {
        println!("  Decisions:");
        for d in &report.decisions {
            println!("    • {}", d);
        }
    } else {
        println!("  Decisions: Unknown");
    }

    if !report.architectures.is_empty() {
        println!("  Architecture:");
        for a in &report.architectures {
            println!("    • {}", a);
        }
    } else {
        println!("  Architecture: Unknown");
    }

    println!("───────────────────────────────────────────────");
    println!("  Evidence:");
    for e in &report.evidence {
        println!("    • {}", e);
    }

    println!("  Path:");
    println!("    {}", report.path.join(" → "));

    println!("  Source: {}", report.source);
    println!("  Confidence: {:.2}", report.confidence);

    if !report.missing.is_empty() {
        println!("───────────────────────────────────────────────");
        println!("  ⚠ Missing Memory:");
        for m in &report.missing {
            println!("    • {}", m);
        }
    }

    println!("═══════════════════════════════════════════════");

    Ok(())
}

pub async fn handle_impact(target_id: &str) -> Result<(), AresError> {
    let store = Store::open(std::path::Path::new(".ares/ares.db"))?;
    let engine = ImpactEngine::new(store);

    let report = engine.analyze(target_id)?;

    println!("═══════════════════════════════════════════════");
    println!("  ARES IMPACT — Analysis Report");
    println!("═══════════════════════════════════════════════");
    println!("  Target: {}", target_id);
    println!("───────────────────────────────────────────────");

    if !report.affected_architecture.is_empty() {
        println!("  Architecture:");
        for a in &report.affected_architecture {
            println!("    • {}", a);
        }
    }

    if !report.affected_decisions.is_empty() {
        println!("  Decisions:");
        for d in &report.affected_decisions {
            println!("    • {}", d);
        }
    }

    if !report.affected_requirements.is_empty() {
        println!("  Requirements:");
        for r in &report.affected_requirements {
            println!("    • {}", r);
        }
    }

    if !report.classifications.is_empty() {
        println!("───────────────────────────────────────────────");
        println!("  Reachability:");
        for (label, reach) in &report.classifications {
            println!("    • {} [{}]", label, reach);
        }
    }

    println!("───────────────────────────────────────────────");
    println!("  Risk Score: {:.2}", report.risk_score);
    println!("═══════════════════════════════════════════════");

    Ok(())
}

pub async fn handle_what_breaks(target_id: &str) -> Result<(), AresError> {
    let store = Store::open(std::path::Path::new(".ares/ares.db"))?;
    let engine = BreakageEngine::new(store);

    let report = engine.what_breaks(target_id)?;

    println!("═══════════════════════════════════════════════");
    println!("  ARES WHAT-BREAKS — Breakage Report");
    println!("═══════════════════════════════════════════════");
    println!("  Target: {}", target_id);
    println!("───────────────────────────────────────────────");

    println!("  Files: {}", report.impacted_files.len());
    for f in &report.impacted_files {
        println!("    • {}", f);
    }

    println!("  Tests:");
    for t in &report.impacted_tests {
        println!("    • {}", t);
    }

    println!("  Runtime Signals:");
    for r in &report.impacted_runtime_signals {
        println!("    • {}", r);
    }

    println!("═══════════════════════════════════════════════");

    Ok(())
}

pub async fn handle_explain_decision(id: &str) -> Result<(), AresError> {
    let store = Store::open(std::path::Path::new(".ares/ares.db"))?;
    let engine = WhyEngine::new(store);

    let report = engine.explain(id)?;

    println!("═══════════════════════════════════════════════");
    println!("  ARES EXPLAIN — Decision Report");
    println!("═══════════════════════════════════════════════");
    println!("  Decision: {}", id);
    println!("  Status: {}", report.status);
    println!("───────────────────────────────────────────────");

    if !report.requirements.is_empty() {
        println!("  Requirements:");
        for r in &report.requirements {
            println!("    • {}", r);
        }
    }

    if !report.architectures.is_empty() {
        println!("  Architecture:");
        for a in &report.architectures {
            println!("    • {}", a);
        }
    }

    println!("  Confidence: {:.2}", report.confidence);
    println!("═══════════════════════════════════════════════");

    Ok(())
}

pub async fn handle_gaps() -> Result<(), AresError> {
    let store = Store::open(std::path::Path::new(".ares/ares.db"))?;
    let engine = GapEngine::new(store);

    let project_id = ProjectId::from("PROJ-001");
    let gaps = engine.detect_gaps(&project_id)?;

    println!("═══════════════════════════════════════════════");
    println!("  ARES GAPS — Memory Gap Detection");
    println!("═══════════════════════════════════════════════");

    if gaps.is_empty() {
        println!("  No memory gaps detected. ✓");
    } else {
        println!("  {} gaps detected:", gaps.len());
        println!("───────────────────────────────────────────────");
        for gap in &gaps {
            println!("  ⚠ {}", gap.gap_description);
            println!("    Missing: {} → {}", gap.from_type, gap.to_type);
            println!("    Node: {}", gap.node_id);
            println!("    Confidence: {:.0}%", gap.confidence * 100.0);
            println!();
        }
    }

    println!("═══════════════════════════════════════════════");

    Ok(())
}
