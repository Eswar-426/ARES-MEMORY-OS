use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollaborationStrategy {
    SingleModel,
    ReasonAndVerify,
    Debate,
    Consensus,
    SpecialistPipeline,
}
