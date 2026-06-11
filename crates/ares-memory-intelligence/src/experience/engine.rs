use super::models::*;
use super::repository::ExperienceRepository;
use crate::episodic::models::Episode;
use ares_core::AresError;
use ares_store::db::Store;
use chrono::Utc;
use tracing::debug;
use uuid::Uuid;

/// Engine for converting events → experiences → lessons → principles.
pub struct ExperienceLearningEngine {
    repo: ExperienceRepository,
}

/// Minimum experiences with similar titles before generating a lesson.
const LESSON_THRESHOLD: u32 = 3;
/// Minimum lessons before generating a principle.
const PRINCIPLE_THRESHOLD: usize = 3;

impl ExperienceLearningEngine {
    pub fn new(store: Store) -> Self {
        Self {
            repo: ExperienceRepository::new(store),
        }
    }

    /// Convert an episode into an experience.
    pub fn record_experience_from_episode(
        &self,
        episode: &Episode,
    ) -> Result<Experience, AresError> {
        let exp_type = if episode.outcome.is_success() {
            ExperienceType::SuccessPattern
        } else {
            ExperienceType::FailurePattern
        };

        // Check if a similar experience already exists
        let similar = self.repo.find_similar_experiences(&episode.title)?;
        if let Some(existing) = similar.first() {
            debug!(
                experience_id = %existing.id,
                "Found similar experience, incrementing frequency"
            );
            self.repo.increment_frequency(&existing.id)?;
            let updated = self.repo.get_experience(&existing.id)?;
            return updated.ok_or_else(|| AresError::not_found("experience", &existing.id));
        }

        let now = Utc::now().timestamp_micros();
        let experience = Experience {
            id: Uuid::now_v7().to_string(),
            episode_id: Some(episode.id.clone()),
            experience_type: exp_type,
            title: episode.title.clone(),
            description: if episode.lessons_learned.is_empty() {
                episode.description.clone()
            } else {
                episode.lessons_learned.join("; ")
            },
            frequency: 1,
            confidence: episode.score.clamp(0.0, 1.0),
            created_at: now,
            updated_at: now,
        };

        debug!(experience_id = %experience.id, "Recording new experience");
        self.repo.insert_experience(&experience)?;
        Ok(experience)
    }

    /// Try to generate a lesson from high-frequency experiences.
    /// Returns the lesson if threshold is met.
    pub fn try_generate_lesson(
        &self,
        experience: &Experience,
    ) -> Result<Option<Lesson>, AresError> {
        if experience.frequency < LESSON_THRESHOLD {
            return Ok(None);
        }

        let similar = self.repo.find_similar_experiences(&experience.title)?;
        let total_frequency: u32 = similar.iter().map(|e| e.frequency).sum();

        if total_frequency < LESSON_THRESHOLD {
            return Ok(None);
        }

        let source_ids: Vec<String> = similar.iter().map(|e| e.id.clone()).collect();
        let avg_confidence =
            similar.iter().map(|e| e.confidence).sum::<f64>() / similar.len().max(1) as f64;

        let lesson = Lesson {
            id: Uuid::now_v7().to_string(),
            title: format!("Lesson: {}", experience.title),
            description: experience.description.clone(),
            source_experience_ids: source_ids,
            confidence: avg_confidence,
            frequency: total_frequency,
            created_at: Utc::now().timestamp_micros(),
        };

        debug!(lesson_id = %lesson.id, frequency = total_frequency, "Generated lesson");
        Ok(Some(lesson))
    }

    /// Try to generate a principle from accumulated lessons.
    pub fn try_generate_principle(
        &self,
        lessons: &[Lesson],
        domain: &str,
    ) -> Result<Option<Principle>, AresError> {
        if lessons.len() < PRINCIPLE_THRESHOLD {
            return Ok(None);
        }

        let avg_confidence =
            lessons.iter().map(|l| l.confidence).sum::<f64>() / lessons.len() as f64;
        let total_evidence: u32 = lessons.iter().map(|l| l.frequency).sum();
        let source_lesson_ids: Vec<String> = lessons.iter().map(|l| l.id.clone()).collect();

        let now = Utc::now().timestamp_micros();
        let principle = Principle {
            id: Uuid::now_v7().to_string(),
            title: format!(
                "Principle: {}",
                lessons
                    .first()
                    .map(|l| l.title.as_str())
                    .unwrap_or("Unknown")
            ),
            description: lessons
                .iter()
                .map(|l| l.description.as_str())
                .collect::<Vec<_>>()
                .join(". "),
            source_lessons: source_lesson_ids,
            evidence_count: total_evidence,
            confidence: avg_confidence,
            domain: domain.into(),
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        debug!(principle_id = %principle.id, evidence = total_evidence, "Generated principle");
        self.repo.insert_principle(&principle)?;
        Ok(Some(principle))
    }

    /// Run the full experience→lesson→principle pipeline for a batch of episodes.
    pub fn process_episodes(&self, episodes: &[Episode]) -> Result<ExperienceReport, AresError> {
        let mut experiences_created = 0u32;
        let mut lessons = Vec::new();

        for episode in episodes {
            let exp = self.record_experience_from_episode(episode)?;
            experiences_created += 1;

            if let Some(lesson) = self.try_generate_lesson(&exp)? {
                lessons.push(lesson);
            }
        }

        let lessons_generated = lessons.len() as u32;
        let mut principles_generated = 0u32;

        if lessons.len() >= PRINCIPLE_THRESHOLD {
            if let Some(_principle) = self.try_generate_principle(&lessons, "general")? {
                principles_generated += 1;
            }
        }

        Ok(ExperienceReport {
            experiences_created,
            lessons_generated,
            principles_generated,
            total_episodes_processed: episodes.len() as u32,
        })
    }

    /// List active principles.
    pub fn list_principles(&self, domain: Option<&str>) -> Result<Vec<Principle>, AresError> {
        self.repo.list_active_principles(domain)
    }

    /// Get experiences by type.
    pub fn get_experiences_by_type(
        &self,
        exp_type: &ExperienceType,
    ) -> Result<Vec<Experience>, AresError> {
        self.repo.find_by_type(exp_type)
    }

    /// Count experiences.
    pub fn count_experiences(&self) -> Result<u64, AresError> {
        self.repo.count_experiences()
    }

    /// Count principles.
    pub fn count_principles(&self) -> Result<u64, AresError> {
        self.repo.count_principles()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episodic::models::EpisodeOutcome;
    use crate::episodic::repository::make_test_episode;
    use crate::test_utils::test_store;

    fn make_engine() -> (ExperienceLearningEngine, tempfile::TempDir) {
        let (store, dir) = test_store();
        (ExperienceLearningEngine::new(store), dir)
    }

    #[test]
    fn record_experience_from_success_episode() {
        let (engine, _dir) = make_engine();
        let ep = make_test_episode("ep_s", EpisodeOutcome::Success);
        let exp = engine.record_experience_from_episode(&ep).unwrap();
        assert_eq!(exp.experience_type, ExperienceType::SuccessPattern);
    }

    #[test]
    fn record_experience_from_failure_episode() {
        let (engine, _dir) = make_engine();
        let ep = make_test_episode("ep_f", EpisodeOutcome::Failure);
        let exp = engine.record_experience_from_episode(&ep).unwrap();
        assert_eq!(exp.experience_type, ExperienceType::FailurePattern);
    }

    #[test]
    fn lesson_not_generated_below_threshold() {
        let (engine, _dir) = make_engine();
        let ep = make_test_episode("ep_1", EpisodeOutcome::Success);
        let exp = engine.record_experience_from_episode(&ep).unwrap();
        assert_eq!(exp.frequency, 1);

        let lesson = engine.try_generate_lesson(&exp).unwrap();
        assert!(lesson.is_none());
    }

    #[test]
    fn process_episodes_creates_experiences() {
        let (engine, _dir) = make_engine();
        let episodes: Vec<Episode> = (0..5)
            .map(|i| {
                let mut ep = make_test_episode(&format!("ep_{}", i), EpisodeOutcome::Success);
                ep.title = format!("Unique title {}", i);
                ep
            })
            .collect();

        let report = engine.process_episodes(&episodes).unwrap();
        assert_eq!(report.total_episodes_processed, 5);
        assert_eq!(report.experiences_created, 5);
    }

    #[test]
    fn principle_not_generated_below_threshold() {
        let (engine, _dir) = make_engine();
        let lessons = vec![Lesson {
            id: "l_1".into(),
            title: "L1".into(),
            description: "D1".into(),
            source_experience_ids: vec![],
            confidence: 0.8,
            frequency: 5,
            created_at: 1000,
        }];
        // Only 1 lesson, threshold is 3
        let principle = engine.try_generate_principle(&lessons, "general").unwrap();
        assert!(principle.is_none());
    }

    #[test]
    fn principle_generated_above_threshold() {
        let (engine, _dir) = make_engine();
        let lessons: Vec<Lesson> = (0..4)
            .map(|i| Lesson {
                id: format!("l_{}", i),
                title: format!("Lesson {}", i),
                description: format!("Description {}", i),
                source_experience_ids: vec![],
                confidence: 0.8,
                frequency: 5,
                created_at: 1000,
            })
            .collect();

        let principle = engine.try_generate_principle(&lessons, "testing").unwrap();
        assert!(principle.is_some());
        let p = principle.unwrap();
        assert_eq!(p.domain, "testing");
        assert!(p.is_active);
    }

    #[test]
    fn list_principles() {
        let (engine, _dir) = make_engine();
        // Generate a principle
        let lessons: Vec<Lesson> = (0..3)
            .map(|i| Lesson {
                id: format!("l_{}", i),
                title: format!("Lesson {}", i),
                description: format!("Desc {}", i),
                source_experience_ids: vec![],
                confidence: 0.9,
                frequency: 10,
                created_at: 1000,
            })
            .collect();
        engine
            .try_generate_principle(&lessons, "deployment")
            .unwrap();

        let principles = engine.list_principles(Some("deployment")).unwrap();
        assert_eq!(principles.len(), 1);
    }

    #[test]
    fn count_experiences_and_principles() {
        let (engine, _dir) = make_engine();
        assert_eq!(engine.count_experiences().unwrap(), 0);
        assert_eq!(engine.count_principles().unwrap(), 0);
    }
}
