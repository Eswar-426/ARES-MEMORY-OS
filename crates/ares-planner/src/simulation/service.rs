use crate::dag::critical_path::CriticalPathAnalysis;
use crate::dag::models::PlanDag;
use crate::models::candidate::PlanCandidate;
use crate::models::simulation::PlanSimulationResult;
use ares_core::AresError;
use chrono::Utc;

pub struct SimulationService;

impl SimulationService {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    /// Simulates a PlanCandidate to estimate cost, duration, success probability, and risk.
    pub fn simulate(&self, candidate: &PlanCandidate) -> Result<PlanSimulationResult, AresError> {
        let dag: PlanDag = serde_json::from_str(&candidate.dag_json)
            .map_err(|e| AresError::validation(format!("Invalid DAG JSON: {}", e)))?;

        // 1. Calculate critical path and duration
        let cp_analysis = CriticalPathAnalysis::calculate(&dag);
        let expected_duration_seconds = cp_analysis.max_duration;

        // 2. Calculate estimated cost
        let expected_cost: f64 = dag.nodes.iter().map(|n| n.cost).sum();

        // 3. Risk and Success Probability (heuristics for now)
        // More nodes/parallelism -> higher risk, lower success prob
        let node_count = dag.nodes.len() as f64;
        let risk_score = if node_count == 0.0 {
            0.0
        } else {
            (node_count * 0.05) + (cp_analysis.parallelism_factor * 0.1)
        };
        let risk_score = risk_score.min(1.0); // clamp at 1.0

        let success_probability = 1.0 - (risk_score * 0.5);

        Ok(PlanSimulationResult {
            plan_id: ares_core::id::PlanId::new(), // Will be mapped to a real PlanId later
            success_probability,
            expected_duration_seconds,
            expected_cost,
            risk_score,
            simulated_at: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::models::{DagEdge, DagNode};
    use ares_core::id::PlanId;

    #[test]
    fn test_simulation_accurate_stubs() {
        let dag = PlanDag {
            nodes: vec![
                DagNode {
                    id: "A".to_string(),
                    title: "".to_string(),
                    estimated_duration: 10.0,
                    cost: 5.0,
                },
                DagNode {
                    id: "B".to_string(),
                    title: "".to_string(),
                    estimated_duration: 20.0,
                    cost: 10.0,
                },
            ],
            edges: vec![DagEdge {
                source: "A".to_string(),
                target: "B".to_string(),
            }],
        };

        let candidate = PlanCandidate::new_test(PlanId::new(), serde_json::to_string(&dag).unwrap());

        let service = SimulationService::new();
        let result = service.simulate(&candidate).unwrap();

        assert_eq!(result.expected_cost, 15.0);
        assert_eq!(result.expected_duration_seconds, 30.0);

        // Node count = 2 -> 2 * 0.05 = 0.1
        // Parallelism factor = 1.0 -> 1.0 * 0.1 = 0.1
        // Total risk = 0.2
        assert_eq!(result.risk_score, 0.2);

        // Success = 1.0 - (0.2 * 0.5) = 0.9
        assert_eq!(result.success_probability, 0.9);
    }
}
