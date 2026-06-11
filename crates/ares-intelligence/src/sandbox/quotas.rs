#[derive(Debug, Clone)]
pub struct Quotas {
    pub max_cost_per_hour: f64,
    pub max_concurrency: u32,
}

impl Default for Quotas {
    fn default() -> Self {
        Self {
            max_cost_per_hour: 10.0,
            max_concurrency: 5,
        }
    }
}
