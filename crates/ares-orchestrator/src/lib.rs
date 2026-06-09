pub mod control;
pub mod runtime;
pub mod events;

use std::sync::Arc;
use ares_store::db::Store;
use ares_core::AresError;

pub struct OrchestratorComponents {
    pub workers_api_state: Arc<control::workers::api::WorkersApiState>,
    pub heartbeat_api_state: Arc<control::heartbeat::api::HeartbeatApiState>,
    pub discovery_api_state: Arc<control::discovery::api::DiscoveryApiState>,
    pub health_api_state: Arc<control::health::api::HealthApiState>,
    pub analytics_api_state: Arc<control::analytics::api::AnalyticsApiState>,
    pub queue_api_state: Arc<runtime::queue::api::QueueApiState>,
    pub dlq_api_state: Arc<runtime::dlq::api::DlqApiState>,
    pub execution_api_state: Arc<runtime::execution::api::ExecutionApiState>,
}

pub fn start_orchestrator(store: Store, config: control::config::OrchestratorConfig) -> Result<OrchestratorComponents, AresError> {
    // 1. Repositories
    let worker_repo = Arc::new(control::workers::repository::WorkerRepository::new(store.clone()));
    let queue_repo = Arc::new(runtime::queue::repository::QueueRepository::new(store.clone()));
    let lease_repo = Arc::new(runtime::leases::repository::LeaseRepository::new(store.clone()));
    let outbox_repo = Arc::new(events::outbox::repository::OutboxRepository::new(store.clone()));
    let dlq_repo = runtime::dlq::repository::DlqRepository::new(store.clone());
    let exec_repo = Arc::new(runtime::execution::repository::ExecutionRepository::new(store.clone()));

    // 2. Services
    let worker_service = control::workers::service::WorkerService::new(
        control::workers::repository::WorkerRepository::new(store.clone())
    );
    let heartbeat_service = control::heartbeat::service::HeartbeatService::new(
        control::workers::repository::WorkerRepository::new(store.clone())
    );
    let discovery_service = control::discovery::routing::DiscoveryService::new(worker_repo.clone());
    let analytics_service = control::analytics::service::AnalyticsService::new(worker_repo.clone());
    let queue_service = runtime::queue::service::QueueService::new(
        runtime::queue::repository::QueueRepository::new(store.clone())
    );
    let dlq_service = runtime::dlq::service::DlqService::new(dlq_repo);
    let exec_service = runtime::execution::service::ExecutionService::new(exec_repo.clone());

    // 3. API States
    let workers_api_state = Arc::new(control::workers::api::WorkersApiState { service: worker_service });
    let heartbeat_api_state = Arc::new(control::heartbeat::api::HeartbeatApiState { service: heartbeat_service });
    let discovery_api_state = Arc::new(control::discovery::api::DiscoveryApiState { service: discovery_service });
    let health_api_state = Arc::new(control::health::api::HealthApiState { worker_repo: worker_repo.clone() });
    let analytics_api_state = Arc::new(control::analytics::api::AnalyticsApiState { service: analytics_service });
    let queue_api_state = Arc::new(runtime::queue::api::QueueApiState { service: queue_service });
    let dlq_api_state = Arc::new(runtime::dlq::api::DlqApiState { service: dlq_service });
    let execution_api_state = Arc::new(runtime::execution::api::ExecutionApiState { service: exec_service });

    // 4. Background Workers
    let heartbeat_monitor = control::heartbeat::monitor::HeartbeatMonitor::new(worker_repo.clone(), config.clone());
    heartbeat_monitor.start();

    let strategy = Arc::new(control::scheduler::strategy::LeastLoadedStrategy);
    let dispatcher = control::scheduler::dispatcher::SchedulerDispatcher::new(worker_repo.clone(), strategy, config.clone());
    dispatcher.start();

    let lease_recovery = runtime::leases::recovery::LeaseRecoveryTask::new(lease_repo.clone(), queue_repo.clone(), config.clone());
    lease_recovery.start();

    let publisher = Arc::new(events::publisher::LocalEventPublisher);
    let outbox_worker = events::outbox::worker::OutboxPublisherWorker::new(outbox_repo.clone(), publisher, config.clone());
    outbox_worker.start();

    Ok(OrchestratorComponents {
        workers_api_state,
        heartbeat_api_state,
        discovery_api_state,
        health_api_state,
        analytics_api_state,
        queue_api_state,
        dlq_api_state,
        execution_api_state,
    })
}
