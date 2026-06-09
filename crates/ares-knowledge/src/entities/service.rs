use ares_store::db::Store;
use uuid::Uuid;
use ares_core::AresError;
use super::models::Entity;
use super::repository::EntityRepository;

pub struct EntityService {
    db: Store,
    repo: EntityRepository,
}

impl EntityService {
    pub fn new(db: Store) -> Self {
        Self {
            db,
            repo: EntityRepository::new(),
        }
    }

    pub async fn create_entity(&self, entity: Entity) -> Result<Entity, AresError> {
        let conn = self.db.get_conn()?;
        self.repo.insert(&conn, &entity)?;
        Ok(entity)
    }

    pub async fn get_entity(&self, id: Uuid) -> Result<Option<Entity>, AresError> {
        let conn = self.db.get_conn()?;
        self.repo.get_by_id(&conn, id)
    }

    pub async fn update_entity(&self, entity: &Entity) -> Result<(), AresError> {
        let conn = self.db.get_conn()?;
        self.repo.update(&conn, entity)
    }

    pub async fn delete_entity(&self, id: Uuid) -> Result<(), AresError> {
        let conn = self.db.get_conn()?;
        self.repo.delete(&conn, id)
    }
}
