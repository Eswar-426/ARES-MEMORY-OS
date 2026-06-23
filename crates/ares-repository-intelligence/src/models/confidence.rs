use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfidence {
    pub capability_confidence: f32,
    pub architecture_confidence: f32,
    pub ownership_confidence: f32,
    pub boundary_confidence: f32,
    pub overall_confidence: f32,
}

impl Default for RepositoryConfidence {
    fn default() -> Self {
        Self {
            capability_confidence: 0.0,
            architecture_confidence: 0.0,
            ownership_confidence: 0.0,
            boundary_confidence: 0.0,
            overall_confidence: 0.0,
        }
    }
}
