#[derive(Debug, Clone)]
pub struct Limits {
    pub max_requests_per_minute: u32,
    pub max_tokens_per_minute: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_requests_per_minute: 100,
            max_tokens_per_minute: 10000,
        }
    }
}
