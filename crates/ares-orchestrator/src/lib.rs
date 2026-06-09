pub mod control;
pub mod events;
pub mod runtime;

use ares_core::AresError;
use ares_store::db::Store;
use std::sync::Arc;

pub struct OrchestratorComponents {
    pub workers_api_state: Arc<control::workers::api::WorkersApiState>,
    pub heartbeat_api_state: Arc<control::heartbeat::api::HeartbeatApiState>,
    pub discovery_api_state: Arc<control::discovery::api::DiscoveryApiState>,
    pub health_api_state: Arc<control::health::api::HealthApiState>,
    pub analytics_api_state: Arc<control::analytics::api::AnalyticsApiState>,
    pub queue_api_state: Arc<runtime::queue::api::QueueApiState>,
    pub dlq_api_state: Arc<runtime::dlq::api::DlqApiState>,
    pub execution_api_state: Arc<runtime::execution::api::ExecutionApiState>,
    pub event_store_api_state: Arc<events::store::api::EventStoreApiState>,
    pub ws_api_state: Arc<events::websocket::api::WsApiState>,
    pub sse_api_state: Arc<events::sse::api::SseApiState>,
}

pub fn start_orchestrator(
    store: Store,
    config: control::config::OrchestratorConfig,
) -> Result<OrchestratorComponents, AresError> {
    // 1. Repositories
    let worker_repo = Arc::new(control::workers::repository::WorkerRepository::new(
        store.clone(),
    ));
    let queue_repo = Arc::new(runtime::queue::repository::QueueRepository::new(
        store.clone(),
    ));
    let lease_repo = Arc::new(runtime::leases::repository::LeaseRepository::new(
        store.clone(),
    ));
    let outbox_repo = Arc::new(events::outbox::repository::OutboxRepository::new(
        store.clone(),
    ));
    let dlq_repo = runtime::dlq::repository::DlqRepository::new(store.clone());
    let exec_repo = Arc::new(runtime::execution::repository::ExecutionRepository::new(
        store.clone(),
    ));

    // 2. Services
    let worker_service = control::workers::service::WorkerService::new(
        control::workers::repository::WorkerRepository::new(store.clone()),
    );
    let heartbeat_service = control::heartbeat::service::HeartbeatService::new(
        control::workers::repository::WorkerRepository::new(store.clone()),
    );
    let discovery_service = control::discovery::routing::DiscoveryService::new(worker_repo.clone());
    let analytics_service = control::analytics::service::AnalyticsService::new(worker_repo.clone());
    let queue_service = runtime::queue::service::QueueService::new(
        runtime::queue::repository::QueueRepository::new(store.clone()),
    );
    let dlq_service = runtime::dlq::service::DlqService::new(dlq_repo);
    let exec_service = runtime::execution::service::ExecutionService::new(exec_repo.clone());

    // 3. API States
    let workers_api_state = Arc::new(control::workers::api::WorkersApiState {
        service: worker_service,
    });
    let heartbeat_api_state = Arc::new(control::heartbeat::api::HeartbeatApiState {
        service: heartbeat_service,
    });
    let discovery_api_state = Arc::new(control::discovery::api::DiscoveryApiState {
        service: discovery_service,
    });
    let health_api_state = Arc::new(control::health::api::HealthApiState {
        worker_repo: worker_repo.clone(),
    });
    let analytics_api_state = Arc::new(control::analytics::api::AnalyticsApiState {
        service: analytics_service,
    });
    let queue_api_state = Arc::new(runtime::queue::api::QueueApiState {
        service: queue_service,
    });
    let dlq_api_state = Arc::new(runtime::dlq::api::DlqApiState {
        service: dlq_service,
    });
    let execution_api_state = Arc::new(runtime::execution::api::ExecutionApiState {
        service: exec_service,
    });

    let event_store_service = events::store::service::EventStoreService::new(
        events::store::repository::EventStoreRepository::new(store.clone()),
        (*outbox_repo).clone(),
    );
    let event_store_api_state = Arc::new(events::store::api::EventStoreApiState {
        service: Arc::new(event_store_service),
    });

    let ws_hub = Arc::new(events::websocket::hub::WsHub::new());
    let ws_api_state = Arc::new(events::websocket::api::WsApiState { hub: ws_hub });

    let sse_api_state = Arc::new(events::sse::api::SseApiState {
        // Shared state goes here if needed later
    });

    // 4. Background Workers
    let heartbeat_monitor =
        control::heartbeat::monitor::HeartbeatMonitor::new(worker_repo.clone(), config.clone());
    heartbeat_monitor.start();

    let strategy = Arc::new(control::scheduler::strategy::LeastLoadedStrategy);
    let dispatcher = control::scheduler::dispatcher::SchedulerDispatcher::new(
        worker_repo.clone(),
        strategy,
        config.clone(),
    );
    dispatcher.start();

    let lease_recovery = runtime::leases::recovery::LeaseRecoveryTask::new(
        lease_repo.clone(),
        queue_repo.clone(),
        config.clone(),
    );
    lease_recovery.start();

    // Event Streaming Backbone Background Workers
    let bus = Arc::new(events::bus::local::LocalEventBus::new(vec![]));
    let dispatcher =
        events::bus::dispatcher::OutboxDispatcher::new(outbox_repo.clone(), bus.clone());
    dispatcher.start();

    // Replay Worker stub
    tokio::spawn(async {
        tracing::info!("ReplayWorker started");
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    });

    // Subscription Monitor stub
    tokio::spawn(async {
        tracing::info!("SubscriptionMonitor started");
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    });

    // Stream Metrics Collector stub
    tokio::spawn(async {
        tracing::info!("StreamMetricsCollector started");
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    });

    Ok(OrchestratorComponents {
        workers_api_state,
        heartbeat_api_state,
        discovery_api_state,
        health_api_state,
        analytics_api_state,
        queue_api_state,
        dlq_api_state,
        execution_api_state,
        event_store_api_state,
        ws_api_state,
        sse_api_state,
    })
}
