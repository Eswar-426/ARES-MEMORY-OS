pub struct TokenEstimator;

impl TokenEstimator {
    /// Provides a rough estimate of token count for a given text.
    /// This uses the standard rule-of-thumb: 1 token ≈ 4 characters for English text.
    /// In the future, this can be replaced with an actual tokenizer like `tiktoken-rs`.
    pub fn estimate_tokens(text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }
        // Basic heuristic: 1 token per 4 chars on average.
        // We add 1 to ensure even a 1-character string counts as 1 token.
        (text.len() / 4) + 1
    }

    /// Estimates the cost of a prompt given the model's cost per input token.
    pub fn estimate_prompt_cost(text: &str, cost_per_input_token: f64) -> f64 {
        let tokens = Self::estimate_tokens(text);
        (tokens as f64) * cost_per_input_token
    }
}
