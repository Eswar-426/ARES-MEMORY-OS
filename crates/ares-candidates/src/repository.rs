use async_trait::async_trait;
use crate::models::{Candidate, CandidateSource, CandidateReview, CandidatePromotion};
use ares_core::{GraphNode, GraphEdge};

#[async_trait]
pub trait CandidateRepository: Send + Sync {
    // Candidates
    async fn insert_candidate(&self, candidate: &Candidate) -> Result<(), String>;
    async fn get_candidate(&self, project_id: &str, id: &str) -> Result<Option<Candidate>, String>;
    async fn update_candidate(&self, candidate: &Candidate) -> Result<(), String>;
    async fn list_candidates(&self, project_id: &str, limit: usize, offset: usize) -> Result<Vec<Candidate>, String>;
    
    // Candidate Sources
    async fn insert_source(&self, source: &CandidateSource) -> Result<(), String>;
    async fn get_sources(&self, project_id: &str, candidate_id: &str) -> Result<Vec<CandidateSource>, String>;
    
    // Reviews
    async fn insert_review(&self, review: &CandidateReview) -> Result<(), String>;
    async fn get_reviews(&self, project_id: &str, candidate_id: &str) -> Result<Vec<CandidateReview>, String>;
    
    // Promotions
    async fn insert_promotion(&self, promotion: &CandidatePromotion) -> Result<(), String>;
    async fn get_promotion(&self, project_id: &str, candidate_id: &str) -> Result<Option<CandidatePromotion>, String>;

    // Transactional Promotion
    async fn promote_candidate(
        &self, 
        candidate: &Candidate, 
        promotion: &CandidatePromotion, 
        node: &GraphNode, 
        edges: &[GraphEdge]
    ) -> Result<(), String>;
}
