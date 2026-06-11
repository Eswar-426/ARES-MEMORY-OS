use std::sync::atomic::{AtomicBool, Ordering};

pub struct KillSwitch {
    is_killed: AtomicBool,
}

impl Default for KillSwitch {
    fn default() -> Self {
        Self::new()
    }
}

impl KillSwitch {
    pub fn new() -> Self {
        Self {
            is_killed: AtomicBool::new(false),
        }
    }

    pub fn activate(&self) {
        self.is_killed.store(true, Ordering::SeqCst);
    }

    pub fn is_active(&self) -> bool {
        self.is_killed.load(Ordering::SeqCst)
    }
}
