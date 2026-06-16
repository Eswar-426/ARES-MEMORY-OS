use crate::memory::models::{MemoryContextPackage, MemoryQuery, RetrievalStrategy};
use ares_core::id::ProjectId;
use ares_decision_intelligence::DecisionStore;
use ares_gap_engine::engine::GapEngine;
use ares_requirements::{RequirementStore, RequirementFilter};
use ares_resolution_engine::engine::ResolutionEngine;
use ares_store::Store;
use std::sync::Arc;

pub trait MemorySource: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_handle(&self, strategy: &RetrievalStrategy) -> bool;
    fn retrieve(&self, store: &Store, query: &MemoryQuery, context: &mut MemoryContextPackage);
}

pub struct MemorySourceRegistry {
    sources: Vec<Arc<dyn MemorySource>>,
    store: Store,
}

impl MemorySourceRegistry {
    pub fn new(store: Store) -> Self {
        Self {
            sources: vec![],
            store,
        }
    }

    pub fn register(&mut self, source: Arc<dyn MemorySource>) {
        self.sources.push(source);
    }

    pub fn execute(&self, strategy: &RetrievalStrategy, query: &MemoryQuery) -> MemoryContextPackage {
        let mut context = MemoryContextPackage::new();

        for source in &self.sources {
            if source.can_handle(strategy) {
                source.retrieve(&self.store, query, &mut context);
            }
        }

        context
    }
}

// Concrete Implementations
pub struct RequirementMemorySource;
impl MemorySource for RequirementMemorySource {
    fn name(&self) -> &'static str {
        "RequirementMemorySource"
    }
    fn can_handle(&self, strategy: &RetrievalStrategy) -> bool {
        matches!(
            strategy,
            RetrievalStrategy::RequirementFocused | RetrievalStrategy::TraceabilityFocused
        )
    }
    fn retrieve(&self, store: &Store, query: &MemoryQuery, context: &mut MemoryContextPackage) {
        let req_store = RequirementStore::new(store.clone());
        let lower_query = query.query.to_lowercase();
        let project_id = ProjectId::from("PROJ-DEFAULT");
        
        let filter = RequirementFilter {
            status: None,
            priority: None,
            requirement_type: None,
            owner: None,
            tag: None,
            since: None,
            until: None,
        };

        if let Ok(all_reqs) = req_store.list(&project_id, filter) {
            for req in all_reqs {
                if req.title.to_lowercase().contains(&lower_query)
                    || req.description.to_lowercase().contains(&lower_query)
                {
                    context.requirements.push(req);
                }
            }
        }
    }
}

pub struct DecisionMemorySource;
impl MemorySource for DecisionMemorySource {
    fn name(&self) -> &'static str {
        "DecisionMemorySource"
    }
    fn can_handle(&self, strategy: &RetrievalStrategy) -> bool {
        matches!(
            strategy,
            RetrievalStrategy::DecisionFocused
                | RetrievalStrategy::GovernanceFocused
                | RetrievalStrategy::TraceabilityFocused
        )
    }
    fn retrieve(&self, store: &Store, query: &MemoryQuery, context: &mut MemoryContextPackage) {
        let dec_store = DecisionStore::new(store.clone());
        let lower_query = query.query.to_lowercase();
        
        if let Ok(all_decs) = dec_store.list() {
            for dec in all_decs {
                if dec.title.to_lowercase().contains(&lower_query)
                    || dec.context.to_lowercase().contains(&lower_query)
                {
                    context.decisions.push(dec.clone());
                    if let Ok(evidence) = dec_store.get_evidence(&dec.id) {
                        context.evidence.extend(evidence);
                    }
                    if let Ok(outcomes) = dec_store.get_outcomes(&dec.id) {
                        context.outcomes.extend(outcomes);
                    }
                }
            }
        }
    }
}

pub struct GapMemorySource;
impl MemorySource for GapMemorySource {
    fn name(&self) -> &'static str {
        "GapMemorySource"
    }
    fn can_handle(&self, strategy: &RetrievalStrategy) -> bool {
        matches!(
            strategy,
            RetrievalStrategy::DebtFocused | RetrievalStrategy::HealthFocused
        )
    }
    fn retrieve(&self, store: &Store, _query: &MemoryQuery, context: &mut MemoryContextPackage) {
        let mut gap_engine = GapEngine::new(Arc::new(store.clone()));
        
        // Register detectors
        gap_engine.register_detector(Box::new(ares_gap_engine::detectors::requirements::RequirementGapDetector));
        gap_engine.register_detector(Box::new(ares_gap_engine::detectors::decisions::DecisionGapDetector));

        let project_id = ProjectId::from("PROJ-DEFAULT");
        
        // Gap engine scan is async
        let rt = tokio::runtime::Runtime::new().unwrap();
        if let Ok(report) = rt.block_on(gap_engine.run_scan(&project_id)) {
            context.health_report = Some(report.clone());
            context.knowledge_debt = Some(report.knowledge_debt.clone());
            context.gaps.extend(report.gaps.clone());
        }
    }
}

pub struct ResolutionMemorySource;
impl MemorySource for ResolutionMemorySource {
    fn name(&self) -> &'static str {
        "ResolutionMemorySource"
    }
    fn can_handle(&self, strategy: &RetrievalStrategy) -> bool {
        matches!(strategy, RetrievalStrategy::DebtFocused)
    }
    fn retrieve(&self, _store: &Store, _query: &MemoryQuery, context: &mut MemoryContextPackage) {
        let res_engine = ResolutionEngine::new();
        
        if let Some(health_report) = &context.health_report {
            let res_report = res_engine.generate_report(health_report);
            context.resolution_plans.extend(res_report.recommended_plans);
        }
    }
}
