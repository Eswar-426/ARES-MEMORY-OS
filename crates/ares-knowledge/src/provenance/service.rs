use ares_store::db::Store;
use ares_core::AresError;
use uuid::Uuid;
use chrono::Utc;
use super::models::ProvenanceRecord;
use super::repository::ProvenanceRepository;

pub struct ProvenanceService {
    db: Store,
    repo: ProvenanceRepository,
}

impl ProvenanceService {
    pub fn new(db: Store) -> Self {
        Self {
            db,
            repo: ProvenanceRepository::new(),
        }
    }

    pub async fn track_entity_provenance(
        &self, 
        entity_id: Uuid, 
        event_id: Uuid, 
        source_type: String, 
        created_by: Option<String>
    ) -> Result<ProvenanceRecord, AresError> {
        let conn = self.db.get_conn()?;
        
        let record = ProvenanceRecord {
            id: Uuid::now_v7(),
            entity_id: Some(entity_id),
            relationship_id: None,
            event_id,
            source_type,
            created_by,
            created_at: Utc::now(),
        };

        self.repo.insert(&conn, &record)?;
        Ok(record)
    }

    pub async fn track_relationship_provenance(
        &self, 
        relationship_id: Uuid, 
        event_id: Uuid, 
        source_type: String, 
        created_by: Option<String>
    ) -> Result<ProvenanceRecord, AresError> {
        let conn = self.db.get_conn()?;
        
        let record = ProvenanceRecord {
            id: Uuid::now_v7(),
            entity_id: None,
            relationship_id: Some(relationship_id),
            event_id,
            source_type,
            created_by,
            created_at: Utc::now(),
        };

        self.repo.insert(&conn, &record)?;
        Ok(record)
    }
}
