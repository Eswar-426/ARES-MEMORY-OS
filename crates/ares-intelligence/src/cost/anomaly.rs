use std::sync::atomic::{AtomicU64, Ordering};

pub struct CostAnomalyDetector {
    historical_hourly_avg: f64,
    anomaly_multiplier: f64,
    current_hour_spend: AtomicU64,
}

impl CostAnomalyDetector {
    pub fn new(historical_hourly_avg: f64, anomaly_multiplier: f64) -> Self {
        Self {
            historical_hourly_avg,
            anomaly_multiplier,
            current_hour_spend: AtomicU64::new(0f64.to_bits()),
        }
    }

    pub fn record_spend(&self, spend: f64) {
        let mut current = self.current_hour_spend.load(Ordering::SeqCst);
        loop {
            let current_f: f64 = f64::from_bits(current);
            let new_f = current_f + spend;
            match self.current_hour_spend.compare_exchange_weak(
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

    pub fn detect_anomaly(&self) -> bool {
        let current_f: f64 = f64::from_bits(self.current_hour_spend.load(Ordering::SeqCst));
        current_f > (self.historical_hourly_avg * self.anomaly_multiplier)
    }
}
