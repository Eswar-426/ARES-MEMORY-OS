use crate::control::config::OrchestratorConfig;
use crate::control::scheduler::strategy::SchedulingStrategy;
use crate::control::workers::repository::WorkerRepository;
use std::sync::Arc;
use tokio::time::interval;
use tracing::info;

pub struct SchedulerDispatcher {
    worker_repo: Arc<WorkerRepository>,
    strategy: Arc<dyn SchedulingStrategy>,
    config: OrchestratorConfig,
}

impl SchedulerDispatcher {
    pub fn new(
        worker_repo: Arc<WorkerRepository>,
        strategy: Arc<dyn SchedulingStrategy>,
        config: OrchestratorConfig,
    ) -> Self {
        Self {
            worker_repo,
            strategy,
            config,
        }
    }

    pub fn start(self) {
        let _worker_repo = self.worker_repo.clone();
        let _strategy = self.strategy.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            info!("Starting SchedulerDispatcher loop");
            let mut ticker = interval(config.scheduler_loop_interval);

            loop {
                ticker.tick().await;

                // Here we would fetch unassigned jobs from the queue and assign them to workers using the strategy.
                // Since the queue is in the runtime plane and we are in the control plane,
                // the dispatcher might need access to a queue repository or an abstract interface.
                // We will hook this up when we implement the queue module.
            }
        });
    }
}
