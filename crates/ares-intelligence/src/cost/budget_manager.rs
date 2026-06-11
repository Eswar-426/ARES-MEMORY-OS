use std::sync::atomic::{AtomicU64, Ordering};

pub struct BudgetManager {
    global_daily_spend_limit: f64,
    max_request_cost: f64,
    current_daily_spend: AtomicU64, // float representation via bits
}

impl BudgetManager {
    pub fn new(global_daily_spend_limit: f64, max_request_cost: f64) -> Self {
        Self {
            global_daily_spend_limit,
            max_request_cost,
            current_daily_spend: AtomicU64::new(0f64.to_bits()),
        }
    }

    pub fn check_request_budget(&self, estimated_cost: f64) -> bool {
        if estimated_cost > self.max_request_cost {
            return false;
        }

        let current_f: f64 = f64::from_bits(self.current_daily_spend.load(Ordering::SeqCst));
        if current_f + estimated_cost > self.global_daily_spend_limit {
            return false;
        }

        true
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
