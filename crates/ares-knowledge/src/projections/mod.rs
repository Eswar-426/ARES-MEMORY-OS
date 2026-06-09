pub trait KnowledgeProjection {
    fn projection_type(&self) -> &str;
    fn build(&self) -> Result<(), ares_core::AresError>;
}

pub struct AgentKnowledgeProjection;
impl KnowledgeProjection for AgentKnowledgeProjection {
    fn projection_type(&self) -> &str {
        "AGENT"
    }
    fn build(&self) -> Result<(), ares_core::AresError> {
        Ok(())
    }
}

pub struct WorkflowKnowledgeProjection;
impl KnowledgeProjection for WorkflowKnowledgeProjection {
    fn projection_type(&self) -> &str {
        "WORKFLOW"
    }
    fn build(&self) -> Result<(), ares_core::AresError> {
        Ok(())
    }
}

pub struct MemoryKnowledgeProjection;
impl KnowledgeProjection for MemoryKnowledgeProjection {
    fn projection_type(&self) -> &str {
        "MEMORY"
    }
    fn build(&self) -> Result<(), ares_core::AresError> {
        Ok(())
    }
}

pub struct DecisionKnowledgeProjection;
impl KnowledgeProjection for DecisionKnowledgeProjection {
    fn projection_type(&self) -> &str {
        "DECISION"
    }
    fn build(&self) -> Result<(), ares_core::AresError> {
        Ok(())
    }
}
