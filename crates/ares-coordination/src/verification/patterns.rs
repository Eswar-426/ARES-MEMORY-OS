use ares_agent_runtime::models::AgentId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VerificationId(pub Uuid);

impl VerificationId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for VerificationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Verification pattern to use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationPattern {
    /// Agent reasons, second agent verifies the reasoning.
    ReasonVerify,
    /// One generates output, another critiques it.
    GenerateCritique,
    /// Planner creates plan, auditor checks for gaps.
    PlanAudit,
    /// Coder produces code, reviewer evaluates.
    CodeReview,
}

/// Result of a verification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VerificationResult {
    Approved { confidence: f64 },
    Rejected { reason: String },
    NeedsRevision { comments: Vec<String> },
}

impl VerificationResult {
    pub fn is_approved(&self) -> bool {
        matches!(self, VerificationResult::Approved { .. })
    }

    pub fn is_rejected(&self) -> bool {
        matches!(self, VerificationResult::Rejected { .. })
    }
}

/// A verification record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    pub id: VerificationId,
    pub pattern: VerificationPattern,
    pub artifact: String,
    pub producer: AgentId,
    pub verifier: AgentId,
    pub result: Option<VerificationResult>,
    pub comments: Vec<String>,
    pub created_at: i64,
    pub resolved_at: Option<i64>,
}

impl Verification {
    pub fn new(
        pattern: VerificationPattern,
        artifact: impl Into<String>,
        producer: AgentId,
        verifier: AgentId,
    ) -> Self {
        Self {
            id: VerificationId::new(),
            pattern,
            artifact: artifact.into(),
            producer,
            verifier,
            result: None,
            comments: Vec::new(),
            created_at: chrono::Utc::now().timestamp(),
            resolved_at: None,
        }
    }

    pub fn submit_review(&mut self, result: VerificationResult, comments: Vec<String>) {
        self.result = Some(result);
        self.comments = comments;
        self.resolved_at = Some(chrono::Utc::now().timestamp());
    }

    pub fn is_resolved(&self) -> bool {
        self.result.is_some()
    }
}
