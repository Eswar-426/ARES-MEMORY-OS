use serde::{Deserialize, Serialize};

/// Type of experience.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperienceType {
    Observation,
    SuccessPattern,
    FailurePattern,
    Optimization,
    Workaround,
}

impl ExperienceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Observation => "observation",
            Self::SuccessPattern => "success_pattern",
            Self::FailurePattern => "failure_pattern",
            Self::Optimization => "optimization",
            Self::Workaround => "workaround",
        }
    }

    pub fn from_str_val(s: &str) -> Self {
        match s {
            "observation" => Self::Observation,
            "success_pattern" => Self::SuccessPattern,
            "failure_pattern" => Self::FailurePattern,
            "optimization" => Self::Optimization,
            "workaround" => Self::Workaround,
            _ => Self::Observation,
        }
    }
}

/// An experience derived from one or more episodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub id: String,
    pub episode_id: Option<String>,
    pub experience_type: ExperienceType,
    pub title: String,
    pub description: String,
    pub frequency: u32,
    pub confidence: f64,
    pub created_at: i64,
    pub updated_at: i64,
}

/// A lesson distilled from multiple related experiences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub title: String,
    pub description: String,
    pub source_experience_ids: Vec<String>,
    pub confidence: f64,
    pub frequency: u32,
    pub created_at: i64,
}

/// A high-level principle derived from repeated lessons.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principle {
    pub id: String,
    pub title: String,
    pub description: String,
    pub source_lessons: Vec<String>,
    pub evidence_count: u32,
    pub confidence: f64,
    pub domain: String,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Report of the experience→lesson→principle pipeline output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceReport {
    pub experiences_created: u32,
    pub lessons_generated: u32,
    pub principles_generated: u32,
    pub total_episodes_processed: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn experience_type_roundtrip() {
        for t in &[
            ExperienceType::Observation,
            ExperienceType::SuccessPattern,
            ExperienceType::FailurePattern,
            ExperienceType::Optimization,
            ExperienceType::Workaround,
        ] {
            assert_eq!(&ExperienceType::from_str_val(t.as_str()), t);
        }
    }

    #[test]
    fn experience_serialization() {
        let exp = Experience {
            id: "exp_1".into(),
            episode_id: Some("ep_1".into()),
            experience_type: ExperienceType::FailurePattern,
            title: "Deployment timeout".into(),
            description: "Services time out during deployment to staging".into(),
            frequency: 3,
            confidence: 0.8,
            created_at: 1000,
            updated_at: 2000,
        };
        let json = serde_json::to_string(&exp).unwrap();
        let back: Experience = serde_json::from_str(&json).unwrap();
        assert_eq!(back.title, "Deployment timeout");
        assert_eq!(back.frequency, 3);
    }

    #[test]
    fn lesson_serialization() {
        let lesson = Lesson {
            id: "les_1".into(),
            title: "Always validate environment first".into(),
            description: "Before deploying, verify all environment variables".into(),
            source_experience_ids: vec!["exp_1".into(), "exp_2".into()],
            confidence: 0.85,
            frequency: 5,
            created_at: 1000,
        };
        let json = serde_json::to_string(&lesson).unwrap();
        let back: Lesson = serde_json::from_str(&json).unwrap();
        assert_eq!(back.source_experience_ids.len(), 2);
    }

    #[test]
    fn principle_serialization() {
        let principle = Principle {
            id: "prin_1".into(),
            title: "Environment verification required before deployment".into(),
            description: "All deployments must pass environment validation".into(),
            source_lessons: vec!["les_1".into()],
            evidence_count: 10,
            confidence: 0.95,
            domain: "deployment".into(),
            is_active: true,
            created_at: 1000,
            updated_at: 2000,
        };
        let json = serde_json::to_string(&principle).unwrap();
        let back: Principle = serde_json::from_str(&json).unwrap();
        assert_eq!(back.domain, "deployment");
        assert!(back.is_active);
    }

    #[test]
    fn experience_report_serialization() {
        let report = ExperienceReport {
            experiences_created: 10,
            lessons_generated: 3,
            principles_generated: 1,
            total_episodes_processed: 50,
        };
        let json = serde_json::to_string(&report).unwrap();
        let back: ExperienceReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.experiences_created, 10);
    }
}
