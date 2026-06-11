use super::checkpoint::SessionCheckpoint;
use super::restore::RestoreEngine;

pub struct RecoveryService {
    restore_engine: RestoreEngine,
}

impl Default for RecoveryService {
    fn default() -> Self {
        Self::new(RestoreEngine)
    }
}

impl RecoveryService {
    #[allow(dead_code)]
    pub fn new(restore_engine: RestoreEngine) -> Self {
        Self { restore_engine }
    }

    #[allow(dead_code)]
    pub fn recover_from_checkpoint(&self, checkpoint: &SessionCheckpoint) -> anyhow::Result<()> {
        self.restore_engine.restore_session(checkpoint)
    }
}
