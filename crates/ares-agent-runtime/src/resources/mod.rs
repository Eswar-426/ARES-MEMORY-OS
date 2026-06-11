use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

pub struct Budgets {
    pub cpu_cores: usize,
    pub memory_mb: u64,
    pub tokens: u64,
    pub api_calls: u64,
    pub concurrency: usize,
}

pub struct ResourceManager {
    total_budgets: Budgets,
    used_cpu_cores: Arc<AtomicUsize>,
    used_memory_mb: Arc<AtomicU64>,
    used_tokens: Arc<AtomicU64>,
    used_api_calls: Arc<AtomicU64>,
    used_concurrency: Arc<AtomicUsize>,
}

impl ResourceManager {
    pub fn new(total_budgets: Budgets) -> Self {
        Self {
            total_budgets,
            used_cpu_cores: Arc::new(AtomicUsize::new(0)),
            used_memory_mb: Arc::new(AtomicU64::new(0)),
            used_tokens: Arc::new(AtomicU64::new(0)),
            used_api_calls: Arc::new(AtomicU64::new(0)),
            used_concurrency: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn acquire_concurrency(&self) -> Result<(), String> {
        let current = self.used_concurrency.load(Ordering::SeqCst);
        if current >= self.total_budgets.concurrency {
            return Err("Concurrency budget exceeded".into());
        }
        self.used_concurrency.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    pub fn release_concurrency(&self) {
        let current = self.used_concurrency.load(Ordering::SeqCst);
        if current > 0 {
            self.used_concurrency.fetch_sub(1, Ordering::SeqCst);
        }
    }

    pub fn consume_tokens(&self, count: u64) -> Result<(), String> {
        let current = self.used_tokens.load(Ordering::SeqCst);
        if current + count > self.total_budgets.tokens {
            return Err("Token budget exceeded".into());
        }
        self.used_tokens.fetch_add(count, Ordering::SeqCst);
        Ok(())
    }

    pub fn consume_api_call(&self) -> Result<(), String> {
        let current = self.used_api_calls.load(Ordering::SeqCst);
        if current + 1 > self.total_budgets.api_calls {
            return Err("API call budget exceeded".into());
        }
        self.used_api_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    pub fn allocate_memory(&self, mb: u64) -> Result<(), String> {
        let current = self.used_memory_mb.load(Ordering::SeqCst);
        if current + mb > self.total_budgets.memory_mb {
            return Err("Memory budget exceeded".into());
        }
        self.used_memory_mb.fetch_add(mb, Ordering::SeqCst);
        Ok(())
    }

    pub fn release_memory(&self, mb: u64) {
        let current = self.used_memory_mb.load(Ordering::SeqCst);
        if current >= mb {
            self.used_memory_mb.fetch_sub(mb, Ordering::SeqCst);
        } else {
            self.used_memory_mb.store(0, Ordering::SeqCst);
        }
    }

    pub fn allocate_cpu(&self, cores: usize) -> Result<(), String> {
        let current = self.used_cpu_cores.load(Ordering::SeqCst);
        if current + cores > self.total_budgets.cpu_cores {
            return Err("CPU budget exceeded".into());
        }
        self.used_cpu_cores.fetch_add(cores, Ordering::SeqCst);
        Ok(())
    }

    pub fn release_cpu(&self, cores: usize) {
        let current = self.used_cpu_cores.load(Ordering::SeqCst);
        if current >= cores {
            self.used_cpu_cores.fetch_sub(cores, Ordering::SeqCst);
        } else {
            self.used_cpu_cores.store(0, Ordering::SeqCst);
        }
    }
}
