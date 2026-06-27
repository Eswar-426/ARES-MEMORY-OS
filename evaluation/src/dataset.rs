use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum FactImportance {
    Required,
    Major,
    Minor,
    Optional,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Claim {
    pub kind: String, // e.g. "decision", "requirement", "owner", "architecture", "function"
    pub id: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Fact {
    pub claim: Claim,
    pub importance: FactImportance,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Evidence {
    pub kind: String,
    pub id: String,
    pub importance: FactImportance,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct EvaluationCase {
    pub engine: String, // e.g. "why", "impact"
    pub target: String, // Target Node ID
    pub facts: Vec<Fact>,
    pub expected_evidence: Vec<Evidence>,
    pub expected_traversal: Vec<String>,
    pub acceptable_answers: Vec<String>,
    pub ideal_answer: Option<String>,
    pub notes: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct EvaluationDataset {
    pub dataset_version: u32,
    pub schema: String,
    pub repository: String,
    pub cases: Vec<EvaluationCase>,
}
