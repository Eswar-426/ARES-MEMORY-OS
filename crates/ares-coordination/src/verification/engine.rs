use ares_agent_runtime::models::AgentId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::patterns::{Verification, VerificationId, VerificationPattern, VerificationResult};

/// Engine for managing verification flows.
pub struct VerificationEngine {
    verifications: Arc<RwLock<HashMap<VerificationId, Verification>>>,
}

impl VerificationEngine {
    pub fn new() -> Self {
        Self {
            verifications: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new verification.
    pub async fn create_verification(
        &self,
        pattern: VerificationPattern,
        artifact: impl Into<String>,
        producer: AgentId,
        verifier: AgentId,
    ) -> VerificationId {
        let verification = Verification::new(pattern, artifact, producer, verifier);
        let id = verification.id;
        self.verifications.write().await.insert(id, verification);
        id
    }

    /// Submit a review for a verification.
    pub async fn submit_review(
        &self,
        verification_id: &VerificationId,
        result: VerificationResult,
        comments: Vec<String>,
    ) -> Result<(), String> {
        let mut verifications = self.verifications.write().await;
        if let Some(verification) = verifications.get_mut(verification_id) {
            verification.submit_review(result, comments);
            Ok(())
        } else {
            Err(format!("Verification {:?} not found", verification_id))
        }
    }

    /// Get the result of a verification.
    pub async fn get_result(&self, verification_id: &VerificationId) -> Option<VerificationResult> {
        self.verifications
            .read()
            .await
            .get(verification_id)
            .and_then(|v| v.result.clone())
    }

    /// Get a verification record.
    pub async fn get_verification(&self, verification_id: &VerificationId) -> Option<Verification> {
        self.verifications
            .read()
            .await
            .get(verification_id)
            .cloned()
    }

    /// Get all pending verifications for a verifier.
    pub async fn get_pending_for_verifier(&self, verifier: &AgentId) -> Vec<Verification> {
        self.verifications
            .read()
            .await
            .values()
            .filter(|v| v.verifier == *verifier && !v.is_resolved())
            .cloned()
            .collect()
    }

    /// Get verification count.
    pub async fn verification_count(&self) -> usize {
        self.verifications.read().await.len()
    }
}

impl Default for VerificationEngine {
    fn default() -> Self {
        Self::new()
    }
}
