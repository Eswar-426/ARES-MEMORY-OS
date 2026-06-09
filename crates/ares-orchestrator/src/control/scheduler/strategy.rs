use crate::control::workers::models::Worker;

pub trait SchedulingStrategy: Send + Sync {
    fn select_worker(
        &self,
        workers: &[Worker],
        workflow_capabilities: &[crate::control::workers::models::WorkerCapability],
    ) -> Option<Worker>;
}

pub struct LeastLoadedStrategy;

impl SchedulingStrategy for LeastLoadedStrategy {
    fn select_worker(
        &self,
        workers: &[Worker],
        _workflow_capabilities: &[crate::control::workers::models::WorkerCapability],
    ) -> Option<Worker> {
        // Find worker with the most available memory
        workers
            .iter()
            .filter(|w| w.status == crate::control::workers::models::WorkerStatus::Online)
            .max_by(|a, b| {
                a.resources
                    .available_memory
                    .cmp(&b.resources.available_memory)
            })
            .cloned()
    }
}

pub struct RoundRobinStrategy {
    // In a real implementation this would hold state, but for now we'll do random or sequential based on time
}

impl SchedulingStrategy for RoundRobinStrategy {
    fn select_worker(
        &self,
        workers: &[Worker],
        _workflow_capabilities: &[crate::control::workers::models::WorkerCapability],
    ) -> Option<Worker> {
        let online_workers: Vec<_> = workers
            .iter()
            .filter(|w| w.status == crate::control::workers::models::WorkerStatus::Online)
            .collect();
        if online_workers.is_empty() {
            None
        } else {
            // Pseudo-random selection for simple round-robin
            let idx = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as usize
                % online_workers.len();
            Some(online_workers[idx].clone())
        }
    }
}

pub struct CapabilityAwareStrategy;

impl SchedulingStrategy for CapabilityAwareStrategy {
    fn select_worker(
        &self,
        workers: &[Worker],
        workflow_capabilities: &[crate::control::workers::models::WorkerCapability],
    ) -> Option<Worker> {
        // Find worker that has ALL required capabilities
        workers
            .iter()
            .filter(|w| w.status == crate::control::workers::models::WorkerStatus::Online)
            .find(|w| {
                workflow_capabilities.iter().all(|req_cap| {
                    w.capabilities.iter().any(|worker_cap| {
                        worker_cap.name == req_cap.name && worker_cap.version == req_cap.version
                    })
                })
            })
            .cloned()
    }
}

pub struct PriorityStrategy;

impl SchedulingStrategy for PriorityStrategy {
    fn select_worker(
        &self,
        workers: &[Worker],
        _workflow_capabilities: &[crate::control::workers::models::WorkerCapability],
    ) -> Option<Worker> {
        // For priority strategy, we might look at worker labels (e.g. "tier"="high-priority")
        workers
            .iter()
            .filter(|w| w.status == crate::control::workers::models::WorkerStatus::Online)
            .max_by(|a, b| {
                let a_tier = a
                    .labels
                    .get("tier")
                    .map(|s| s.as_str())
                    .unwrap_or("default");
                let b_tier = b
                    .labels
                    .get("tier")
                    .map(|s| s.as_str())
                    .unwrap_or("default");
                a_tier.cmp(b_tier)
            })
            .cloned()
    }
}
