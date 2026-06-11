use std::sync::atomic::{AtomicU64, Ordering};

pub struct CostMetrics {
    total_tokens: AtomicU64,
    total_spend: AtomicU64, // float representation
}

impl Default for CostMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl CostMetrics {
    pub fn new() -> Self {
        Self {
            total_tokens: AtomicU64::new(0),
            total_spend: AtomicU64::new(0f64.to_bits()),
        }
    }

    pub fn record_cost(&self, tokens: u64, cost: f64) {
        self.total_tokens.fetch_add(tokens, Ordering::SeqCst);

        let mut current = self.total_spend.load(Ordering::SeqCst);
        loop {
            let current_f: f64 = f64::from_bits(current);
            let new_f = current_f + cost;
            match self.total_spend.compare_exchange_weak(
                current,
                new_f.to_bits(),
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => break,
                Err(x) => current = x,
            }
        }
    }

    pub fn get_total_tokens(&self) -> u64 {
        self.total_tokens.load(Ordering::SeqCst)
    }

    pub fn get_total_spend(&self) -> f64 {
        f64::from_bits(self.total_spend.load(Ordering::SeqCst))
    }
}
