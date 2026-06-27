use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum FactKind {
    Requirement,
    Decision,
    File,
    Function,
    Test,
    Architecture,
    Owner,
    Dependency,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, PartialOrd)]
pub struct CanonicalFact {
    pub schema_version: u16,
    pub kind: FactKind,
    pub id: String,
    pub confidence: f64,
}

// Ensure Eq and Ord logic handles floats safely since we only sort
impl Eq for CanonicalFact {}
impl Ord for CanonicalFact {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}
