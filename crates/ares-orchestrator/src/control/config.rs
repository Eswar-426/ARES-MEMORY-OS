use std::time::Duration;

#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    pub heartbeat_timeout: Duration,
    pub heartbeat_check_interval: Duration,
    pub scheduler_loop_interval: Duration,
    pub lease_renewal_interval: Duration,
    pub default_lease_duration: Duration,
    pub outbox_poll_interval: Duration,
    pub dlq_retry_interval: Duration,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            heartbeat_timeout: Duration::from_secs(30),
            heartbeat_check_interval: Duration::from_secs(10),
            scheduler_loop_interval: Duration::from_secs(5),
            lease_renewal_interval: Duration::from_secs(15),
            default_lease_duration: Duration::from_secs(60),
            outbox_poll_interval: Duration::from_secs(2),
            dlq_retry_interval: Duration::from_secs(60),
        }
    }
}
