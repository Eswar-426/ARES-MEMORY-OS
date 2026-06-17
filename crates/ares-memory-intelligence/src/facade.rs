use std::sync::Arc;
use ares_core::AresError;
use ares_knowledge_graph::impact::ImpactReport;
use ares_memory_evolution::models::EvolutionTimeline;
use crate::assembler::MemoryContextAssembler;

/// MemoryFacade shields the API and external consumers from internal orchestration logic.
#[derive(Clone)]
pub struct MemoryFacade {
    assembler: Arc<MemoryContextAssembler>,
}

impl MemoryFacade {
    pub fn new(
        assembler: Arc<MemoryContextAssembler>,
    ) -> Self {
        Self {
            assembler,
        }
    }

    pub fn why(&self, entity_id: &str) -> Result<serde_json::Value, AresError> {
        let result = self.assembler.graph.why_does_this_exist(entity_id)?;
        Ok(serde_json::json!({
            "entity": entity_id,
            "requirements": result.requirements,
            "decisions": result.decisions,
            "evidence": result.evidence
        }))
    }

    pub fn who(&self, entity_id: &str) -> Result<serde_json::Value, AresError> {
        let result = self.assembler.graph.who_owns_this(entity_id)?;
        Ok(serde_json::json!({
            "entity": entity_id,
            "owners": result.owners,
            "approvers": result.approvers,
            "decisions": result.decisions
        }))
    }
    
    pub fn approval(&self, entity_id: &str) -> Result<serde_json::Value, AresError> {
        let result = self.assembler.graph.who_owns_this(entity_id)?;
        Ok(serde_json::json!({
            "entity": entity_id,
            "approvers": result.approvers,
            "decisions": result.decisions
        }))
    }

    pub fn evidence(&self, entity_id: &str) -> Result<serde_json::Value, AresError> {
        let evidence = self.assembler.graph.what_evidence_supports_this(entity_id)?;
        Ok(serde_json::json!({
            "entity": entity_id,
            "evidence": evidence
        }))
    }

    pub fn replacement(&self, entity_id: &str) -> Result<serde_json::Value, AresError> {
        // Mocked supersession for now as EvolutionTimeline handles replacement at higher level
        Ok(serde_json::json!({
            "entity": entity_id,
            "superseded_by": null
        }))
    }



    pub fn impact(&self, entity_id: &str) -> Result<serde_json::Value, AresError> {
        let report = self.assembler.graph.what_breaks_if_changed(entity_id)?;
        Ok(serde_json::json!({
            "total_score": report.total_score,
            "risk_level": report.risk_level,
            "impacted_nodes": report.impacted_nodes
        }))
    }

    pub fn evolution(&self, entity_id: &str) -> Result<serde_json::Value, AresError> {
        let timeline = self.assembler.evolution.how_has_this_evolved(entity_id)?;
        Ok(serde_json::json!({
            "entity": entity_id,
            "revisions": timeline.revisions.len() // Mocking the deep serialization for now
        }))
    }

    pub fn context(&self, entity_id: &str) -> Result<serde_json::Value, AresError> {
        self.assembler.get_entity_full_context(entity_id)
    }

    pub fn get_assembler(&self) -> Arc<MemoryContextAssembler> {
        self.assembler.clone()
    }
}
