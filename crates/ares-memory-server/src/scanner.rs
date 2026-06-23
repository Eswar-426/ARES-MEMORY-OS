use ares_core::AresError;
use std::path::Path;

pub struct RepositoryScanner;

impl RepositoryScanner {
    pub fn scan(_path: &Path) -> Result<(), AresError> {
        // Orchestrate ares-scanner
        Ok(())
    }
}
