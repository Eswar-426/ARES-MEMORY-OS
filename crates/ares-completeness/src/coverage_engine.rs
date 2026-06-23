use crate::models::{CoverageMetrics, HierarchySegment};

pub struct CoverageEngine;

impl Default for CoverageEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CoverageEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_coverage(&self, segments: &[HierarchySegment]) -> CoverageMetrics {
        let mut req_total = 0;
        let mut req_covered = 0;

        let mut dec_total = 0;
        let mut dec_covered = 0;

        let mut arch_total = 0;
        let mut arch_covered = 0;

        let mut code_total = 0;
        let mut code_covered = 0;

        let mut test_total = 0;
        let mut test_covered = 0;

        let mut run_total = 0;
        let mut run_covered = 0;

        for seg in segments {
            let is_covered = |target: &str| -> bool {
                seg.state != crate::models::TopologyState::Orphaned
                    && seg.state != crate::models::TopologyState::Disconnected
                    && !seg.missing_downstream.contains(&target.to_string())
            };

            match seg.node_type.as_str() {
                "Requirement" => {
                    req_total += 1;
                    if is_covered("Decision") {
                        req_covered += 1;
                    }
                }
                "Decision" => {
                    dec_total += 1;
                    if is_covered("Architecture") {
                        dec_covered += 1;
                    }
                }
                "Architecture" => {
                    arch_total += 1;
                    if is_covered("Code") {
                        arch_covered += 1;
                    }
                }
                "File" | "Function" | "Method" | "Class" | "Struct" | "Enum" | "Trait"
                | "Module" => {
                    code_total += 1;
                    if is_covered("Test") {
                        code_covered += 1;
                    }
                }
                "Test" => {
                    test_total += 1;
                    if is_covered("RuntimeSignal") {
                        test_covered += 1;
                    }
                }
                "RuntimeSignal" => {
                    run_total += 1;
                    if is_covered("Outcome") {
                        run_covered += 1;
                    }
                }
                _ => {}
            }
        }

        let calc_pct = |covered: usize, total: usize| -> f32 {
            if total == 0 {
                0.0
            } else {
                (covered as f32 / total as f32) * 100.0
            }
        };

        CoverageMetrics {
            requirement_coverage: calc_pct(req_covered, req_total),
            decision_coverage: calc_pct(dec_covered, dec_total),
            architecture_coverage: calc_pct(arch_covered, arch_total),
            code_coverage: calc_pct(code_covered, code_total),
            test_coverage: calc_pct(test_covered, test_total),
            runtime_coverage: calc_pct(run_covered, run_total),
        }
    }
}
