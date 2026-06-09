use super::repository::SnapshotRepository;
use ares_core::AresError;

pub struct SnapshotService {
    repo: SnapshotRepository,
}

impl SnapshotService {
    pub fn new(repo: SnapshotRepository) -> Self {
        Self { repo }
    }

    pub fn take_snapshot(&self, aggregate_id: &str, aggregate_type: &str, version: u32, snapshot_data: &str) -> Result<(), AresError> {
        self.repo.insert_snapshot(aggregate_id, aggregate_type, version, snapshot_data)?;
        Ok(())
    }
}
