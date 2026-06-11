use crate::models::{AgentId, AgentRole};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Capability {
    CodeGeneration,
    TestGeneration,
    ArchitectureDesign,
    CodeReview,
    SecurityAnalysis,
    Documentation,
    WebBrowsing,
    DatabaseAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilitySet {
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderSelection {
    OpenAI(String),
    Anthropic(String),
    Google(String),
    Local(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub role: AgentRole,
    pub capabilities: CapabilitySet,
    pub provider: ProviderSelection,
    pub system_prompt: String,
    pub temperature: f32,
}

pub struct AgentInstance {
    pub id: AgentId,
    pub config: AgentConfig,
}

pub struct AgentRegistry {
    templates: HashMap<AgentRole, AgentConfig>,
    active_instances: HashMap<AgentId, AgentInstance>,
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            active_instances: HashMap::new(),
        }
    }

    pub fn register_template(&mut self, config: AgentConfig) {
        self.templates.insert(config.role.clone(), config);
    }

    pub fn get_template(&self, role: &AgentRole) -> Option<AgentConfig> {
        self.templates.get(role).cloned()
    }

    pub fn allocate_agent(&mut self, role: &AgentRole) -> Result<AgentId, String> {
        if let Some(config) = self.get_template(role) {
            let id = AgentId::new();
            let instance = AgentInstance { id, config };
            self.active_instances.insert(id, instance);
            Ok(id)
        } else {
            Err(format!("No template found for role {:?}", role))
        }
    }

    pub fn release_agent(&mut self, id: &AgentId) -> Result<(), String> {
        if self.active_instances.remove(id).is_some() {
            Ok(())
        } else {
            Err("Agent not found".into())
        }
    }

    pub fn health_check(&self, id: &AgentId) -> bool {
        self.active_instances.contains_key(id)
    }

    pub fn get_capabilities(&self, id: &AgentId) -> Option<CapabilitySet> {
        self.active_instances
            .get(id)
            .map(|inst| inst.config.capabilities.clone())
    }
}
