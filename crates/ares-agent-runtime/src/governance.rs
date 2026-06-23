use crate::models::ExecutionStatus;
use ares_core::{NodeId, ProjectId};
use ares_governance::approval::ApprovalEngine;
use ares_governance::events::GovernanceEventPublisher;
use ares_governance::models::{
    GovernanceOutcome, PolicyDefinition, PolicyExemption, PolicyVersion,
};
use ares_governance::mutation_simulator::{GraphMutation, VirtualGraphProvider};
use ares_governance::strict_evaluation::StrictEvaluationEngine;

pub struct GovernanceInterceptor {
    publisher: GovernanceEventPublisher,
    approval_engine: ApprovalEngine,
}

impl GovernanceInterceptor {
    pub fn new(publisher: GovernanceEventPublisher, approval_engine: ApprovalEngine) -> Self {
        Self {
            publisher,
            approval_engine,
        }
    }

    pub async fn pre_flight_check(
        &self,
        project_id: &ProjectId,
        workflow_id: &str,
        node_id: &NodeId,
        mutation: GraphMutation,
        base_nodes: Vec<ares_core::types::node::GraphNode>,
        base_edges: Vec<ares_core::types::node::GraphEdge>,
        policies: &[(PolicyDefinition, PolicyVersion)],
        exemptions: &[PolicyExemption],
    ) -> Result<ExecutionStatus, String> {
        let provider = VirtualGraphProvider::new(base_nodes, base_edges, mutation);

        match StrictEvaluationEngine::evaluate(project_id, node_id, provider, policies, exemptions)
        {
            Ok(result) => match result.outcome {
                GovernanceOutcome::Allow => Ok(ExecutionStatus::Running),
                GovernanceOutcome::Warn => {
                    self.publisher.publish(
                        ares_governance::models::GovernanceEvent::ViolationDetected {
                            project_id: project_id.to_string(),
                            workflow_id: workflow_id.to_string(),
                            violations: result.violations,
                        },
                    );
                    Ok(ExecutionStatus::Running)
                }
                GovernanceOutcome::RequireApproval => {
                    let request = self
                        .approval_engine
                        .create_request(
                            project_id.as_ref(),
                            workflow_id,
                            result.violations.clone(),
                        )
                        .await
                        .map_err(|e| e.to_string())?;

                    self.publisher.publish(
                        ares_governance::models::GovernanceEvent::ApprovalRequested { request },
                    );
                    Ok(ExecutionStatus::AwaitingApproval)
                }
                GovernanceOutcome::Block => {
                    self.publisher
                        .publish(ares_governance::models::GovernanceEvent::Blocked {
                            project_id: project_id.to_string(),
                            workflow_id: workflow_id.to_string(),
                            reason: "Blocked by governance policy".to_string(),
                            violations: result.violations,
                        });
                    Ok(ExecutionStatus::GovernanceBlocked)
                }
            },
            Err(e) => Err(format!("Pre-flight simulation failed: {}", e)),
        }
    }
}
