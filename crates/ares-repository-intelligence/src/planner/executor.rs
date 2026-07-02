use crate::core::capabilities::Capability;
use crate::core::context::RepositoryContext;
use crate::core::engine::{EngineExecutionResult, EngineInput};
use crate::core::errors::EngineResult;
use crate::planner::registry::EngineRegistry;

pub struct EngineExecutor<'a> {
    registry: &'a EngineRegistry,
}

impl<'a> EngineExecutor<'a> {
    pub fn new(registry: &'a EngineRegistry) -> Self {
        Self { registry }
    }

    #[tracing::instrument(name = "EngineExecutor::execute", skip(self, context, input), fields(capability = ?capability))]
    pub async fn execute(
        &self,
        capability: &Capability,
        context: &RepositoryContext,
        input: EngineInput,
    ) -> EngineResult<Vec<EngineExecutionResult>> {
        let engine_ids = self.registry.resolve_capabilities(capability);
        let mut results = Vec::new();

        for engine_id in engine_ids {
            if let Some(engine) = self.registry.get_engine(&engine_id) {
                let start = std::time::Instant::now();
                tracing::info!("Executing engine {:?}", engine_id);

                let result = engine.execute(context, input.clone()).await?;

                tracing::info!(
                    duration_ms = start.elapsed().as_millis(),
                    engine = ?engine_id,
                    "Engine execution completed"
                );

                results.push(result);
            }
        }

        Ok(results)
    }
}
