use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolCategory {
    Filesystem,
    Shell,
    Browser,
    Search,
    Memory,
    Knowledge,
    LLM,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub category: ToolCategory,
    pub parameters_schema: String, // JSON schema representation
}

#[derive(Debug, Clone)]
pub struct ToolPermissions {
    pub allow_network: bool,
    pub allow_filesystem_read: bool,
    pub allow_filesystem_write: bool,
    pub allow_shell: bool,
}

pub struct ToolRegistry {
    tools: HashMap<String, ToolDefinition>,
    permissions_map: HashMap<String, ToolPermissions>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            permissions_map: HashMap::new(),
        }
    }

    pub fn register_tool(&mut self, tool: ToolDefinition, permissions: ToolPermissions) {
        let name = tool.name.clone();
        self.tools.insert(name.clone(), tool);
        self.permissions_map.insert(name, permissions);
    }

    pub fn get_tool(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.get(name)
    }

    pub fn check_permissions(&self, name: &str) -> Option<&ToolPermissions> {
        self.permissions_map.get(name)
    }

    pub fn get_tools_by_category(&self, category: &ToolCategory) -> Vec<&ToolDefinition> {
        self.tools
            .values()
            .filter(|t| &t.category == category)
            .collect()
    }
}
