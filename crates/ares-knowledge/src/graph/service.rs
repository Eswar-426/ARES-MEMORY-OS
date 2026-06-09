use super::repository::GraphRepository;
use ares_core::AresError;
use ares_store::db::Store;

pub struct GraphService {
    db: Store,
    repo: GraphRepository,
}

impl GraphService {
    pub fn new(db: Store) -> Self {
        Self {
            db,
            repo: GraphRepository::new(),
        }
    }

    pub async fn get_statistics(&self) -> Result<serde_json::Value, AresError> {
        let conn = self.db.get_conn()?;
        self.repo.get_statistics(&conn)
    }
}
