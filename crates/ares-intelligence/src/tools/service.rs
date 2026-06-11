use super::registry::ToolRegistry;
use super::selector::ToolSelector;

pub struct ToolIntelligenceService {
    registry: ToolRegistry,
    selector: ToolSelector,
}

impl Default for ToolIntelligenceService {
    fn default() -> Self {
        Self::new(ToolRegistry::default(), ToolSelector::default())
    }
}

impl ToolIntelligenceService {
    #[allow(dead_code)]
    pub fn new(registry: ToolRegistry, selector: ToolSelector) -> Self {
        Self { registry, selector }
    }

    #[allow(dead_code)]
    pub fn resolve_tools_for_task(&self, prompt: &str) -> anyhow::Result<Vec<String>> {
        let caps = self.selector.select_tools(prompt)?;
        let mut tool_names = Vec::new();

        for cap in caps {
            let tools = self.registry.get_by_capability(cap);
            if let Some(tool) = tools.first() {
                tool_names.push(tool.name.clone());
            }
        }

        Ok(tool_names)
    }
}
