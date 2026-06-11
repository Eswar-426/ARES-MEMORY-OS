use crate::models::{AgentId, MissionId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub key: String,
    pub value: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Default)]
pub struct MissionMemory {
    pub mission_id: MissionId,
    items: HashMap<String, MemoryItem>,
}

impl MissionMemory {
    pub fn new(mission_id: MissionId) -> Self {
        Self {
            mission_id,
            items: HashMap::new(),
        }
    }

    pub fn store(&mut self, key: String, value: String) {
        self.items.insert(
            key.clone(),
            MemoryItem {
                key,
                value,
                timestamp: chrono::Utc::now().timestamp(),
            },
        );
    }

    pub fn retrieve(&self, key: &str) -> Option<String> {
        self.items.get(key).map(|i| i.value.clone())
    }
}

#[derive(Debug, Clone, Default)]
pub struct AgentMemory {
    pub agent_id: AgentId,
    short_term: HashMap<String, MemoryItem>,
    long_term_references: Vec<String>,
}

impl AgentMemory {
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            short_term: HashMap::new(),
            long_term_references: Vec::new(),
        }
    }

    pub fn store_short_term(&mut self, key: String, value: String) {
        self.short_term.insert(
            key.clone(),
            MemoryItem {
                key,
                value,
                timestamp: chrono::Utc::now().timestamp(),
            },
        );
    }

    pub fn get_short_term(&self, key: &str) -> Option<String> {
        self.short_term.get(key).map(|i| i.value.clone())
    }

    pub fn add_long_term_reference(&mut self, ref_id: String) {
        self.long_term_references.push(ref_id);
    }
}

#[derive(Debug, Clone, Default)]
pub struct SharedContext {
    pub scope_id: String,
    context_data: HashMap<String, String>,
}

impl SharedContext {
    pub fn new(scope_id: String) -> Self {
        Self {
            scope_id,
            context_data: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        self.context_data.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.context_data.get(key).cloned()
    }
}
