use super::super::entities::models::Entity;
use super::repository::SearchRepository;
use ares_core::AresError;
use ares_store::db::Store;

pub struct SearchService {
    db: Store,
    repo: SearchRepository,
}

impl SearchService {
    pub fn new(db: Store) -> Self {
        Self {
            db,
            repo: SearchRepository::new(),
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Entity>, AresError> {
        let conn = self.db.get_conn()?;
        self.repo.search_entities(&conn, query)
    }
}
