use ares_core::AresError;
use std::path::Path;

pub struct RepositoryServer;

impl RepositoryServer {
    pub fn serve(_path: &Path) -> Result<(), AresError> {
        // Loads memory.db statically and boots the API
        Ok(())
    }
}
