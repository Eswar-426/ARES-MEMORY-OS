use super::capability::ToolCapability;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub capability: ToolCapability,
}

pub struct ToolRegistry {
    tools: HashMap<String, ToolDefinition>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn register(&mut self, tool: ToolDefinition) {
        self.tools.insert(tool.name.clone(), tool);
    }

    #[allow(dead_code)]
    pub fn get_by_capability(&self, cap: ToolCapability) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .filter(|t| t.capability == cap)
            .cloned()
            .collect()
    }
}
