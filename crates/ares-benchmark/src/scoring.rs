/// The Hybrid Scoring Engine calculates the success rate of a benchmark run.

#[derive(Debug, Clone, Default)]
pub struct HybridScorer {}

impl HybridScorer {
    pub fn new() -> Self {
        Self {}
    }

    /// Calculate the hybrid success score (0.0 to 100.0)
    ///
    /// Weights:
    /// - Compile Pass: 25%
    /// - Tests Pass: 25%
    /// - Architecture Rules: 20%
    /// - Task Completion: 15%
    /// - LLM Judge: 15%
    pub fn score_run(
        &self,
        compile_pass: bool,
        tests_pass: bool,
        arch_score: f64,      // 0.0 to 1.0
        task_completion: f64, // 0.0 to 1.0
        llm_judge: f64,       // 0.0 to 1.0
    ) -> f64 {
        let mut total = 0.0;

        if compile_pass {
            total += 25.0;
        }

        if tests_pass {
            total += 25.0;
        }

        total += arch_score * 20.0;
        total += task_completion * 15.0;
        total += llm_judge * 15.0;

        total
    }
}
