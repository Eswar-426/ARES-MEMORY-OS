use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

pub struct MetricsService {
    counters: Mutex<HashMap<String, AtomicU64>>,
}

impl Default for MetricsService {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsService {
    pub fn new() -> Self {
        Self {
            counters: Mutex::new(HashMap::new()),
        }
    }

    pub fn record_counter(&self, metric_name: &str, value: u64) {
        let mut map = self.counters.lock().unwrap();
        let counter = map
            .entry(metric_name.to_string())
            .or_insert_with(|| AtomicU64::new(0));
        counter.fetch_add(value, Ordering::SeqCst);
    }

    pub fn get_counter(&self, metric_name: &str) -> u64 {
        if let Some(c) = self.counters.lock().unwrap().get(metric_name) {
            c.load(Ordering::SeqCst)
        } else {
            0
        }
    }
}
