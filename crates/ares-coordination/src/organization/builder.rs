use ares_agent_runtime::models::{AgentId, AgentRole};

use super::hierarchy::AgentHierarchy;
use super::node::AgentNode;
use super::team::{AgentTeam, TeamStrategy};

/// Fluent API for constructing agent organizations.
pub struct OrganizationBuilder {
    name: String,
    leader_role: Option<AgentRole>,
    teams: Vec<TeamSpec>,
}

struct TeamSpec {
    name: String,
    roles: Vec<AgentRole>,
    strategy: TeamStrategy,
}

impl OrganizationBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            leader_role: None,
            teams: Vec::new(),
        }
    }

    /// Set the leader (root) role for the organization.
    pub fn with_leader(mut self, role: AgentRole) -> Self {
        self.leader_role = Some(role);
        self
    }

    /// Add a team with default Parallel strategy.
    pub fn add_team(mut self, name: impl Into<String>, roles: &[AgentRole]) -> Self {
        self.teams.push(TeamSpec {
            name: name.into(),
            roles: roles.to_vec(),
            strategy: TeamStrategy::Parallel,
        });
        self
    }

    /// Add a team with a specific strategy.
    pub fn add_team_with_strategy(
        mut self,
        name: impl Into<String>,
        roles: &[AgentRole],
        strategy: TeamStrategy,
    ) -> Self {
        self.teams.push(TeamSpec {
            name: name.into(),
            roles: roles.to_vec(),
            strategy,
        });
        self
    }

    /// Build the organization hierarchy.
    pub fn build(self) -> AgentHierarchy {
        let mut hierarchy = AgentHierarchy::new(&self.name);

        // Create leader node
        let leader_role = self.leader_role.unwrap_or(AgentRole::CEO);
        let leader_id = AgentId::new();
        let leader_node = AgentNode::new(leader_id, leader_role);
        hierarchy.add_node(leader_node);

        // Create teams and their members
        for team_spec in &self.teams {
            let mut team =
                AgentTeam::new(&team_spec.name, "").with_strategy(team_spec.strategy.clone());

            let mut first_member = true;
            for role in &team_spec.roles {
                let agent_id = AgentId::new();
                let node = AgentNode::new(agent_id, role.clone())
                    .with_team(team.id)
                    .with_parent(leader_id);

                team.add_member(agent_id, role.clone());

                if first_member {
                    team.set_leader(agent_id);
                    first_member = false;
                }

                hierarchy.add_node(node);
                // Set parent-child relationship
                let _ = hierarchy.set_parent(agent_id, leader_id);
            }

            team.activate();
            hierarchy.add_team(team);
        }

        hierarchy
    }
}

/// Build a standard software engineering organization.
pub fn build_standard_engineering_org() -> AgentHierarchy {
    OrganizationBuilder::new("ARES Engineering")
        .with_leader(AgentRole::CEO)
        .add_team_with_strategy(
            "planning",
            &[
                AgentRole::Architect,
                AgentRole::Planner,
                AgentRole::Researcher,
            ],
            TeamStrategy::Pipeline,
        )
        .add_team_with_strategy(
            "engineering",
            &[AgentRole::Coder, AgentRole::Tester, AgentRole::Reviewer],
            TeamStrategy::Pipeline,
        )
        .add_team("operations", &[AgentRole::Operator, AgentRole::Security])
        .build()
}

/// Build a minimal research organization.
pub fn build_research_org() -> AgentHierarchy {
    OrganizationBuilder::new("ARES Research")
        .with_leader(AgentRole::Architect)
        .add_team("research", &[AgentRole::Researcher, AgentRole::Researcher])
        .add_team("validation", &[AgentRole::Tester, AgentRole::Reviewer])
        .build()
}
