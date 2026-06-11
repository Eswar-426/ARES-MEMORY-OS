use super::models::*;
use ares_core::AresError;
use ares_store::db::Store;
use rusqlite::params;

/// SQLite-backed repository for experience reports, lessons, and principles.
pub struct ExperienceRepository {
    store: Store,
}

impl ExperienceRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    // ---- Experience Reports ----

    /// Insert an experience report.
    pub fn insert_experience(&self, exp: &Experience) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO experience_reports (id, episode_id, experience_type, title, description,
             lesson, principle_id, frequency, confidence, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, '', NULL, ?6, ?7, ?8, ?9)",
            params![
                exp.id,
                exp.episode_id,
                exp.experience_type.as_str(),
                exp.title,
                exp.description,
                exp.frequency,
                exp.confidence,
                exp.created_at,
                exp.updated_at,
            ],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    /// Get an experience by ID.
    pub fn get_experience(&self, id: &str) -> Result<Option<Experience>, AresError> {
        let conn = self.store.get_conn()?;
        let result = conn.query_row(
            "SELECT id, episode_id, experience_type, title, description, frequency, confidence,
                    created_at, updated_at
             FROM experience_reports WHERE id = ?1",
            params![id],
            |row| {
                Ok(Experience {
                    id: row.get(0)?,
                    episode_id: row.get(1)?,
                    experience_type: ExperienceType::from_str_val(&row.get::<_, String>(2)?),
                    title: row.get(3)?,
                    description: row.get(4)?,
                    frequency: row.get(5)?,
                    confidence: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        );
        match result {
            Ok(e) => Ok(Some(e)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AresError::db(e)),
        }
    }

    /// Find experiences by type.
    pub fn find_by_type(&self, exp_type: &ExperienceType) -> Result<Vec<Experience>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, episode_id, experience_type, title, description, frequency, confidence,
                        created_at, updated_at
                 FROM experience_reports WHERE experience_type = ?1
                 ORDER BY frequency DESC LIMIT 100",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![exp_type.as_str()], |row| {
                Ok(Experience {
                    id: row.get(0)?,
                    episode_id: row.get(1)?,
                    experience_type: ExperienceType::from_str_val(&row.get::<_, String>(2)?),
                    title: row.get(3)?,
                    description: row.get(4)?,
                    frequency: row.get(5)?,
                    confidence: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })
            .map_err(AresError::db)?;

        let mut experiences = Vec::new();
        for row in rows {
            experiences.push(row.map_err(AresError::db)?);
        }
        Ok(experiences)
    }

    /// Find experiences by title similarity.
    pub fn find_similar_experiences(&self, title: &str) -> Result<Vec<Experience>, AresError> {
        let conn = self.store.get_conn()?;
        let pattern = format!("%{}%", title);
        let mut stmt = conn
            .prepare(
                "SELECT id, episode_id, experience_type, title, description, frequency, confidence,
                        created_at, updated_at
                 FROM experience_reports WHERE title LIKE ?1
                 ORDER BY frequency DESC LIMIT 50",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![pattern], |row| {
                Ok(Experience {
                    id: row.get(0)?,
                    episode_id: row.get(1)?,
                    experience_type: ExperienceType::from_str_val(&row.get::<_, String>(2)?),
                    title: row.get(3)?,
                    description: row.get(4)?,
                    frequency: row.get(5)?,
                    confidence: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })
            .map_err(AresError::db)?;

        let mut experiences = Vec::new();
        for row in rows {
            experiences.push(row.map_err(AresError::db)?);
        }
        Ok(experiences)
    }

    /// Increment frequency of an experience.
    pub fn increment_frequency(&self, id: &str) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let now = chrono::Utc::now().timestamp_micros();
        let rows = conn
            .execute(
                "UPDATE experience_reports SET frequency = frequency + 1, updated_at = ?1 WHERE id = ?2",
                params![now, id],
            )
            .map_err(AresError::db)?;
        if rows == 0 {
            return Err(AresError::not_found("experience_report", id));
        }
        Ok(())
    }

    // ---- Principles ----

    /// Insert a principle.
    pub fn insert_principle(&self, p: &Principle) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let lessons_json = serde_json::to_string(&p.source_lessons)
            .map_err(|e| AresError::Serialization(e.to_string()))?;

        conn.execute(
            "INSERT INTO memory_principles (id, title, description, source_lessons, evidence_count,
             confidence, domain, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                p.id,
                p.title,
                p.description,
                lessons_json,
                p.evidence_count,
                p.confidence,
                p.domain,
                p.is_active as i32,
                p.created_at,
                p.updated_at,
            ],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    /// Get a principle by ID.
    pub fn get_principle(&self, id: &str) -> Result<Option<Principle>, AresError> {
        let conn = self.store.get_conn()?;
        let result = conn.query_row(
            "SELECT id, title, description, source_lessons, evidence_count, confidence,
                    domain, is_active, created_at, updated_at
             FROM memory_principles WHERE id = ?1",
            params![id],
            |row| Self::row_to_principle(row),
        );
        match result {
            Ok(p) => Ok(Some(p)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AresError::db(e)),
        }
    }

    /// List active principles by domain.
    pub fn list_active_principles(
        &self,
        domain: Option<&str>,
    ) -> Result<Vec<Principle>, AresError> {
        let conn = self.store.get_conn()?;
        let (sql, param_values): (String, Vec<Box<dyn rusqlite::types::ToSql>>) =
            if let Some(d) = domain {
                (
                    "SELECT id, title, description, source_lessons, evidence_count, confidence,
                        domain, is_active, created_at, updated_at
                 FROM memory_principles WHERE is_active = 1 AND domain = ?1
                 ORDER BY confidence DESC LIMIT 100"
                        .into(),
                    vec![Box::new(d.to_string())],
                )
            } else {
                (
                    "SELECT id, title, description, source_lessons, evidence_count, confidence,
                        domain, is_active, created_at, updated_at
                 FROM memory_principles WHERE is_active = 1
                 ORDER BY confidence DESC LIMIT 100"
                        .into(),
                    vec![],
                )
            };

        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql).map_err(AresError::db)?;
        let rows = stmt
            .query_map(params_ref.as_slice(), |row| Self::row_to_principle(row))
            .map_err(AresError::db)?;

        let mut principles = Vec::new();
        for row in rows {
            principles.push(row.map_err(AresError::db)?);
        }
        Ok(principles)
    }

    /// Count experiences.
    pub fn count_experiences(&self) -> Result<u64, AresError> {
        let conn = self.store.get_conn()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM experience_reports", [], |row| {
                row.get(0)
            })
            .map_err(AresError::db)?;
        Ok(count as u64)
    }

    /// Count principles.
    pub fn count_principles(&self) -> Result<u64, AresError> {
        let conn = self.store.get_conn()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM memory_principles", [], |row| {
                row.get(0)
            })
            .map_err(AresError::db)?;
        Ok(count as u64)
    }

    fn row_to_principle(row: &rusqlite::Row<'_>) -> Result<Principle, rusqlite::Error> {
        let lessons_str: String = row.get(3)?;
        let is_active_int: i32 = row.get(7)?;
        Ok(Principle {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            source_lessons: serde_json::from_str(&lessons_str).unwrap_or_default(),
            evidence_count: row.get(4)?,
            confidence: row.get(5)?,
            domain: row.get(6)?,
            is_active: is_active_int != 0,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    }
}

#[cfg(test)]
pub fn make_test_experience(id: &str, exp_type: ExperienceType) -> Experience {
    let now = chrono::Utc::now().timestamp_micros();
    Experience {
        id: id.into(),
        episode_id: Some("ep_1".into()),
        experience_type: exp_type,
        title: format!("Experience {}", id),
        description: format!("Description for {}", id),
        frequency: 1,
        confidence: 0.7,
        created_at: now,
        updated_at: now,
    }
}

#[cfg(test)]
pub fn make_test_principle(id: &str) -> Principle {
    let now = chrono::Utc::now().timestamp_micros();
    Principle {
        id: id.into(),
        title: format!("Principle {}", id),
        description: format!("Description for {}", id),
        source_lessons: vec!["les_1".into()],
        evidence_count: 3,
        confidence: 0.85,
        domain: "general".into(),
        is_active: true,
        created_at: now,
        updated_at: now,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_store;

    #[test]
    fn insert_and_get_experience() {
        let (store, _dir) = test_store();
        let repo = ExperienceRepository::new(store);
        let exp = make_test_experience("exp_1", ExperienceType::FailurePattern);
        repo.insert_experience(&exp).unwrap();

        let fetched = repo.get_experience("exp_1").unwrap().unwrap();
        assert_eq!(fetched.experience_type, ExperienceType::FailurePattern);
    }

    #[test]
    fn get_nonexistent_experience() {
        let (store, _dir) = test_store();
        let repo = ExperienceRepository::new(store);
        assert!(repo.get_experience("nope").unwrap().is_none());
    }

    #[test]
    fn find_by_type() {
        let (store, _dir) = test_store();
        let repo = ExperienceRepository::new(store);
        repo.insert_experience(&make_test_experience(
            "e_fp",
            ExperienceType::FailurePattern,
        ))
        .unwrap();
        repo.insert_experience(&make_test_experience(
            "e_sp",
            ExperienceType::SuccessPattern,
        ))
        .unwrap();

        let results = repo.find_by_type(&ExperienceType::FailurePattern).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "e_fp");
    }

    #[test]
    fn find_similar_experiences() {
        let (store, _dir) = test_store();
        let repo = ExperienceRepository::new(store);
        let mut exp = make_test_experience("e_deploy", ExperienceType::FailurePattern);
        exp.title = "Deployment timeout issue".into();
        repo.insert_experience(&exp).unwrap();

        let results = repo.find_similar_experiences("timeout").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn increment_frequency() {
        let (store, _dir) = test_store();
        let repo = ExperienceRepository::new(store);
        repo.insert_experience(&make_test_experience("e_freq", ExperienceType::Observation))
            .unwrap();

        repo.increment_frequency("e_freq").unwrap();
        let fetched = repo.get_experience("e_freq").unwrap().unwrap();
        assert_eq!(fetched.frequency, 2);
    }

    #[test]
    fn increment_nonexistent_fails() {
        let (store, _dir) = test_store();
        let repo = ExperienceRepository::new(store);
        assert!(repo.increment_frequency("nope").is_err());
    }

    #[test]
    fn insert_and_get_principle() {
        let (store, _dir) = test_store();
        let repo = ExperienceRepository::new(store);
        let p = make_test_principle("p_1");
        repo.insert_principle(&p).unwrap();

        let fetched = repo.get_principle("p_1").unwrap().unwrap();
        assert_eq!(fetched.title, "Principle p_1");
        assert!(fetched.is_active);
    }

    #[test]
    fn list_active_principles() {
        let (store, _dir) = test_store();
        let repo = ExperienceRepository::new(store);
        repo.insert_principle(&make_test_principle("p_active"))
            .unwrap();

        let mut inactive = make_test_principle("p_inactive");
        inactive.is_active = false;
        repo.insert_principle(&inactive).unwrap();

        let active = repo.list_active_principles(None).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, "p_active");
    }

    #[test]
    fn list_active_principles_by_domain() {
        let (store, _dir) = test_store();
        let repo = ExperienceRepository::new(store);

        let mut p1 = make_test_principle("p_deploy");
        p1.domain = "deployment".into();
        repo.insert_principle(&p1).unwrap();

        let mut p2 = make_test_principle("p_general");
        p2.domain = "general".into();
        repo.insert_principle(&p2).unwrap();

        let results = repo.list_active_principles(Some("deployment")).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "p_deploy");
    }

    #[test]
    fn count_experiences_and_principles() {
        let (store, _dir) = test_store();
        let repo = ExperienceRepository::new(store);
        assert_eq!(repo.count_experiences().unwrap(), 0);
        assert_eq!(repo.count_principles().unwrap(), 0);

        repo.insert_experience(&make_test_experience("e_c", ExperienceType::Observation))
            .unwrap();
        repo.insert_principle(&make_test_principle("p_c")).unwrap();
        assert_eq!(repo.count_experiences().unwrap(), 1);
        assert_eq!(repo.count_principles().unwrap(), 1);
    }
}
