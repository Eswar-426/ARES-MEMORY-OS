use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct ContinuityMetrics {
    // Retention Metrics (0-100)
    pub architecture: f64,      // 20%
    pub requirements: f64,      // 20%
    pub features: f64,          // 20%
    pub decisions: f64,         // 15%
    pub bug_history: f64,       // 10%
    pub recovery_accuracy: f64, // 15%

    // Efficiency Metrics
    pub context_compression_ratio: f64,
    pub context_transfer_efficiency: f64,
    pub token_savings: usize,
}

impl ContinuityMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn final_score(&self) -> f64 {
        self.architecture * 0.20
            + self.requirements * 0.20
            + self.features * 0.20
            + self.decisions * 0.15
            + self.bug_history * 0.10
            + self.recovery_accuracy * 0.15
    }
}

pub struct ContinuityReport {
    // Maps a scenario name to its metric results
    scenario_metrics: HashMap<String, ContinuityMetrics>,
}

impl Default for ContinuityReport {
    fn default() -> Self {
        Self::new()
    }
}

impl ContinuityReport {
    pub fn new() -> Self {
        Self {
            scenario_metrics: HashMap::new(),
        }
    }

    pub fn record_metrics(&mut self, scenario_name: &str, metrics: ContinuityMetrics) {
        self.scenario_metrics
            .insert(scenario_name.to_string(), metrics);
    }

    pub fn overall_score(&self) -> f64 {
        let mut ares_total = 0.0;
        let mut count = 0;
        for (run_name, metrics) in &self.scenario_metrics {
            if run_name.ends_with(" (Ares)") {
                ares_total += metrics.final_score();
                count += 1;
            }
        }
        if count > 0 {
            ares_total / count as f64
        } else {
            0.0
        }
    }

    pub fn print_report(&self) {
        println!("\n=======================================================");
        println!("🚀 ARES MEMORY OS - CONTINUITY BENCHMARK REPORT 🚀");
        println!("=======================================================\n");

        if self.scenario_metrics.is_empty() {
            println!("No scenarios evaluated.");
            println!("=======================================================\n");
            return;
        }

        let mut ares_total = 0.0;
        let mut baseline_total = 0.0;
        let mut count = 0;

        // Group by base scenario name (removing " (Baseline)" or " (Ares)")
        let mut grouped: HashMap<String, (Option<ContinuityMetrics>, Option<ContinuityMetrics>)> =
            HashMap::new();

        for (run_name, metrics) in &self.scenario_metrics {
            if run_name.ends_with(" (Baseline)") {
                let base_name = run_name.replace(" (Baseline)", "");
                let entry = grouped.entry(base_name).or_insert((None, None));
                entry.0 = Some(metrics.clone());
            } else if run_name.ends_with(" (Ares)") {
                let base_name = run_name.replace(" (Ares)", "");
                let entry = grouped.entry(base_name).or_insert((None, None));
                entry.1 = Some(metrics.clone());
            }
        }

        for (scenario, (baseline_opt, ares_opt)) in &grouped {
            println!("📌 Scenario: {}", scenario);

            let base_score = baseline_opt
                .as_ref()
                .map(|m| m.final_score())
                .unwrap_or(0.0);
            let ares_score = ares_opt.as_ref().map(|m| m.final_score()).unwrap_or(0.0);

            baseline_total += base_score;
            ares_total += ares_score;
            count += 1;

            println!("   Baseline Retention: {:>6.2}%", base_score);
            println!(
                "   ARES Retention:     {:>6.2}% | {}",
                ares_score,
                Self::get_grade(ares_score)
            );

            let diff = ares_score - base_score;
            let sign = if diff >= 0.0 { "+" } else { "" };
            println!("   Improvement:        {}{:>5.2}%", sign, diff);

            if let Some(metrics) = ares_opt {
                println!("   --- ARES Retention Metrics ---");
                println!("   Architecture (20%): {:>6.2}%", metrics.architecture);
                println!("   Requirements (20%): {:>6.2}%", metrics.requirements);
                println!("   Features     (20%): {:>6.2}%", metrics.features);
                println!("   Decisions    (15%): {:>6.2}%", metrics.decisions);
                println!("   Bug History  (10%): {:>6.2}%", metrics.bug_history);
                println!("   Recovery     (15%): {:>6.2}%", metrics.recovery_accuracy);
            }
            println!();
        }

        let ares_avg = if count > 0 {
            ares_total / count as f64
        } else {
            0.0
        };
        let base_avg = if count > 0 {
            baseline_total / count as f64
        } else {
            0.0
        };
        let overall_diff = ares_avg - base_avg;
        let overall_sign = if overall_diff >= 0.0 { "+" } else { "" };

        println!("=======================================================");
        println!("🔥 BASELINE CONTINUITY: {:>6.2}%", base_avg);
        println!(
            "🔥 ARES CONTINUITY:     {:>6.2}% | {}",
            ares_avg,
            Self::get_grade(ares_avg)
        );
        println!(
            "🔥 TOTAL IMPROVEMENT:   {}{:>5.2}%",
            overall_sign, overall_diff
        );
        println!("=======================================================\n");
    }

    fn get_grade(score: f64) -> &'static str {
        if score >= 90.0 {
            "🟢 EXCELLENT"
        } else if score >= 80.0 {
            "🟡 GOOD"
        } else if score >= 70.0 {
            "🟠 ACCEPTABLE"
        } else if score >= 60.0 {
            "🔴 WEAK"
        } else {
            "💀 FAILED"
        }
    }
}
