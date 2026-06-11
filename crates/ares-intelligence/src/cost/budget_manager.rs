use std::sync::atomic::{AtomicU64, Ordering};

pub struct BudgetManager {
    global_daily_spend_limit: f64,
    current_daily_spend: AtomicU64, // float representation via bits
}

impl BudgetManager {
    pub fn new(global_daily_spend_limit: f64) -> Self {
        Self {
            global_daily_spend_limit,
            current_daily_spend: AtomicU64::new(0f64.to_bits()),
        }
    }

    pub fn add_spend(&self, cost: f64) {
        let mut current = self.current_daily_spend.load(Ordering::SeqCst);
        loop {
            let current_f: f64 = f64::from_bits(current);
            let new_f = current_f + cost;
            match self.current_daily_spend.compare_exchange_weak(
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

    pub fn is_budget_exceeded(&self) -> bool {
        let current_f: f64 = f64::from_bits(self.current_daily_spend.load(Ordering::SeqCst));
        current_f >= self.global_daily_spend_limit
    }
}
