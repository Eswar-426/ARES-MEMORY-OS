use crate::models::{LinkTarget, LinkTargetType};
use ares_core::{AresError, RequirementId, RequirementLinkId};
use ares_store::db::Store;
use ares_traceability::{EdgeProvider, TraceabilityEdge, TraceTargetType};
use chrono::Utc;
use rusqlite::params;

pub struct RequirementEdgeProvider {
    store: Store,
}

impl RequirementEdgeProvider {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn link(
        &self,
        source: &RequirementId,
        target: LinkTarget,
        relationship: &str,
        created_by: Option<&str>,
    ) -> Result<RequirementLinkId, AresError> {
        let conn = self.store.get_conn()?;
        let link_id = RequirementLinkId::new();
        let now = Utc::now().timestamp_micros();
        
        let target_type_str = match target.target_type() {
            LinkTargetType::Requirement => "requirement",
            LinkTargetType::Decision => "decision",
            LinkTargetType::Architecture => "architecture",
            LinkTargetType::Code => "code",
        };

        conn.execute(
            "INSERT INTO requirement_links (
                id, source_requirement_id, target_id, target_type, relationship, created_at, created_by
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                link_id.as_str(),
                source.as_str(),
                target.target_id(),
                target_type_str,
                relationship,
                now,
                created_by,
            ],
        )
        .map_err(AresError::db)?;

        Ok(link_id)
    }

    pub fn unlink(
        &self,
        source: &RequirementId,
        target: &LinkTarget,
    ) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let target_type_str = match target.target_type() {
            LinkTargetType::Requirement => "requirement",
            LinkTargetType::Decision => "decision",
            LinkTargetType::Architecture => "architecture",
            LinkTargetType::Code => "code",
        };

        conn.execute(
            "DELETE FROM requirement_links 
             WHERE source_requirement_id = ?1 AND target_id = ?2 AND target_type = ?3",
            params![
                source.as_str(),
                target.target_id(),
                target_type_str,
            ],
        )
        .map_err(AresError::db)?;

        Ok(())
    }
}

impl EdgeProvider for RequirementEdgeProvider {
    fn edges(&self) -> Result<Vec<TraceabilityEdge>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare("SELECT source_requirement_id, target_id, target_type, relationship FROM requirement_links")
            .map_err(AresError::db)?;

        let edges = stmt.query_map([], |row| {
            let source_id: String = row.get(0)?;
            let target_id: String = row.get(1)?;
            let target_type_str: String = row.get(2)?;
            let relationship: String = row.get(3)?;
            
            let target_type = match target_type_str.as_str() {
                "requirement" => TraceTargetType::Requirement,
                "decision" => TraceTargetType::Decision,
                "architecture" => TraceTargetType::Architecture,
                "code" => TraceTargetType::Code,
                other => TraceTargetType::Unknown(other.to_string()),
            };

            Ok(TraceabilityEdge {
                source_id,
                target_id,
                target_type,
                relationship,
            })
        }).map_err(AresError::db)?
        .collect::<Result<Vec<_>, _>>().map_err(AresError::db)?;

        Ok(edges)
    }
}
