use crate::core::capabilities::Capability;

pub struct ExecutionNode {
    pub capability: Capability,
    pub dependencies: Vec<Capability>, // Wait for these to finish before running this
}

pub struct ExecutionGraph {
    pub nodes: Vec<ExecutionNode>,
}

impl Default for ExecutionGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionGraph {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }
}
