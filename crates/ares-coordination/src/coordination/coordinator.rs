use ares_agent_runtime::models::AgentRole;

use crate::consensus::ConsensusEngine;
use crate::debate::DebateEngine;
use crate::delegation::DelegationEngine;
use crate::governor::SafetyGovernor;
use crate::messaging::MessageBus;
use crate::organization::hierarchy::AgentHierarchy;
use crate::organization::OrganizationBuilder;
use crate::organizational_learning::OrgLearningEngine;
use crate::reputation::ReputationTracker;
use crate::resource_manager::CoordinationResourceManager;
use crate::shared_memory::SharedWorkspace;
use crate::swarm::SwarmEngine;
use crate::verification::VerificationEngine;

/// The master coordinator that connects all subsystems.
pub struct AgentCoordinator {
    pub message_bus: MessageBus,
    pub workspace: SharedWorkspace,
    pub delegation: DelegationEngine,
    pub consensus: ConsensusEngine,
    pub debate: DebateEngine,
    pub verification: VerificationEngine,
    pub reputation: ReputationTracker,
    pub swarm: SwarmEngine,
    pub resources: CoordinationResourceManager,
    pub governor: SafetyGovernor,
    pub learning: OrgLearningEngine,
    pub organization: Option<AgentHierarchy>,
}

impl AgentCoordinator {
    pub fn new(resources: CoordinationResourceManager, governor: SafetyGovernor) -> Self {
        Self {
            message_bus: MessageBus::new(),
            workspace: SharedWorkspace::new(),
            delegation: DelegationEngine::new(),
            consensus: ConsensusEngine::new(),
            debate: DebateEngine::new(),
            verification: VerificationEngine::new(),
            reputation: ReputationTracker::new(),
            swarm: SwarmEngine::new(),
            resources,
            governor,
            learning: OrgLearningEngine::new(),
            organization: None,
        }
    }

    /// Build an organization for a goal.
    pub fn build_organization(&mut self, goal: &str) -> &AgentHierarchy {
        let org = OrganizationBuilder::new(format!("org-{}", &goal[..goal.len().min(20)]))
            .with_leader(AgentRole::CEO)
            .add_team(
                "planning",
                &[
                    AgentRole::Architect,
                    AgentRole::Planner,
                    AgentRole::Researcher,
                ],
            )
            .add_team(
                "engineering",
                &[AgentRole::Coder, AgentRole::Tester, AgentRole::Reviewer],
            )
            .build();
        self.organization = Some(org);
        self.organization.as_ref().unwrap()
    }

    /// Get the current organization.
    pub fn get_organization(&self) -> Option<&AgentHierarchy> {
        self.organization.as_ref()
    }

    /// Check if the coordinator is ready (has an organization and resources).
    pub fn is_ready(&self) -> bool {
        self.organization.is_some()
    }
}

impl Default for AgentCoordinator {
    fn default() -> Self {
        Self::new(
            CoordinationResourceManager::default(),
            SafetyGovernor::default(),
        )
    }
}
