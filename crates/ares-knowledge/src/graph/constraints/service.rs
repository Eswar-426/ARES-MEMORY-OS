use ares_store::db::Store;
use ares_core::AresError;
use super::models::Constraint;

pub struct ConstraintService {
    db: Store,
}

impl ConstraintService {
    pub fn new(db: Store) -> Self {
        Self { db }
    }

    pub async fn get_all_constraints(&self) -> Result<Vec<Constraint>, AresError> {
        let _conn = self.db.get_conn()?;
        // Mock returning constraints from `graph_constraints` table
        Ok(vec![])
    }
}
