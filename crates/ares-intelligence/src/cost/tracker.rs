#[derive(Debug, Clone, Default)]
pub struct CostTracker {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
    pub total_cost_usd: f64,
}

impl CostTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_usage(&mut self, prompt: usize, completion: usize, cost_usd: f64) {
        self.prompt_tokens += prompt;
        self.completion_tokens += completion;
        self.total_tokens += prompt + completion;
        self.total_cost_usd += cost_usd;
    }
}
