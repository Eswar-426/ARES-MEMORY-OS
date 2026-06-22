pub struct RiskEngine;

impl RiskEngine {
    pub fn new() -> Self {
        Self
    }

    /// Calculates a deterministic risk score for touching a component based on its downstream memory surface area.
    /// Formula:
    /// Risk Score = (Requirements * 0.25) + (Decisions * 0.25) + (Architecture Nodes * 0.20) + (Files * 0.15) + (Tests * 0.15)
    /// Normalized: 0.0 -> 1.0
    /// 
    /// Note: This is versioned and frozen. Changes require an ADR.
    pub fn calculate_risk_score(
        &self,
        req_count: usize,
        dec_count: usize,
        arch_count: usize,
        file_count: usize,
        test_count: usize,
    ) -> f32 {
        // We cap the maximum values to achieve a normalization between 0.0 and 1.0
        // Assumptions for capping: 10 Reqs, 10 Decs, 10 Archs, 20 Files, 20 Tests max out the risk.
        let req_score = ((req_count as f32).min(10.0) / 10.0) * 0.25;
        let dec_score = ((dec_count as f32).min(10.0) / 10.0) * 0.25;
        let arch_score = ((arch_count as f32).min(10.0) / 10.0) * 0.20;
        let file_score = ((file_count as f32).min(20.0) / 20.0) * 0.15;
        let test_score = ((test_count as f32).min(20.0) / 20.0) * 0.15;

        let total_score = req_score + dec_score + arch_score + file_score + test_score;
        total_score.clamp(0.0, 1.0)
    }
}
