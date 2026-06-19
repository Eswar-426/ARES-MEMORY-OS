use ares_core::{AresError, NodeId, ProjectId};
use crate::compliance_engine::ComplianceEngine;
use crate::models::{ComplianceViolation, EnforcementAction, MemoryRiskLevel, PolicyDefinition, PolicyVersion, GovernanceOutcome};
use crate::mutation_simulator::VirtualGraphProvider;

pub struct StrictEvaluationEngine;

pub struct StrictEvaluationResult {
    pub allowed: bool,
    pub outcome: GovernanceOutcome,
    pub risk_level: MemoryRiskLevel,
    pub violations: Vec<ComplianceViolation>,
}

impl StrictEvaluationEngine {
    pub fn evaluate(
        project_id: &ProjectId,
        node_id: &NodeId,
        provider: VirtualGraphProvider,
        policies: &[(PolicyDefinition, PolicyVersion)],
        exemptions: &[crate::models::PolicyExemption],
    ) -> Result<StrictEvaluationResult, AresError> {
        let engine = ComplianceEngine::new(provider);
        let results = engine.evaluate_node(project_id, node_id, policies, exemptions)?;
        
        let mut violations = Vec::new();
        for r in results {
            violations.extend(r.violations);
        }
        
        let has_blocking = violations.iter().any(|v| v.enforcement == EnforcementAction::Block);
        let has_approval = violations.iter().any(|v| v.enforcement == EnforcementAction::RequireApproval);
        
        // Strict Evaluation Logic (Phase 8G & 8H rule scope)
        let (risk_level, outcome) = if has_blocking {
            (MemoryRiskLevel::MemoryCritical, GovernanceOutcome::Block)
        } else if has_approval {
            (MemoryRiskLevel::MemoryRisk, GovernanceOutcome::RequireApproval)
        } else if !violations.is_empty() {
            (MemoryRiskLevel::MemoryWarning, GovernanceOutcome::Warn)
        } else {
            (MemoryRiskLevel::MemorySafe, GovernanceOutcome::Allow)
        };
        
        Ok(StrictEvaluationResult {
            allowed: !has_blocking && !has_approval, // Only true if it doesn't block or pause workflow
            outcome,
            risk_level,
            violations,
        })
    }
}
