use super::models::Relationship;
use super::repository::RelationshipRepository;
use ares_core::AresError;
use ares_store::db::Store;
use uuid::Uuid;

pub struct RelationshipService {
    db: Store,
    repo: RelationshipRepository,
}

impl RelationshipService {
    pub fn new(db: Store) -> Self {
        Self {
            db,
            repo: RelationshipRepository::new(),
        }
    }

    pub async fn create_relationship(&self, rel: Relationship) -> Result<Relationship, AresError> {
        let conn = self.db.get_conn()?;
        self.repo.insert(&conn, &rel)?;
        Ok(rel)
    }

    pub async fn delete_relationship(&self, id: Uuid) -> Result<(), AresError> {
        let conn = self.db.get_conn()?;
        self.repo.delete(&conn, id)
    }
}
