use crate::models::AgentId;
use crate::workflow::MissionNode;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchedulingStrategy {
    Priority,
    RoundRobin,
    CapabilityAware,
    LeastLoaded,
}

#[derive(Debug, Clone)]
pub struct TaskQueueItem {
    pub task: MissionNode,
    pub priority: u32,
}

pub struct AgentScheduler {
    strategy: SchedulingStrategy,
    queue: VecDeque<TaskQueueItem>,
    agent_loads: HashMap<AgentId, usize>,
}

impl AgentScheduler {
    pub fn new(strategy: SchedulingStrategy) -> Self {
        Self {
            strategy,
            queue: VecDeque::new(),
            agent_loads: HashMap::new(),
        }
    }

    pub fn enqueue_task(&mut self, task: MissionNode, priority: u32) {
        let item = TaskQueueItem { task, priority };
        match self.strategy {
            SchedulingStrategy::Priority => {
                // Insert maintaining priority order (highest first)
                let pos = self
                    .queue
                    .binary_search_by(|t| t.priority.cmp(&item.priority).reverse())
                    .unwrap_or_else(|e| e);
                self.queue.insert(pos, item);
            }
            _ => {
                self.queue.push_back(item);
            }
        }
    }

    pub fn dequeue_task(&mut self) -> Option<MissionNode> {
        self.queue.pop_front().map(|i| i.task)
    }

    pub fn update_agent_load(&mut self, agent: AgentId, load: usize) {
        self.agent_loads.insert(agent, load);
    }

    pub fn select_agent(&self, available_agents: &[AgentId]) -> Option<AgentId> {
        if available_agents.is_empty() {
            return None;
        }

        match self.strategy {
            SchedulingStrategy::LeastLoaded => available_agents
                .iter()
                .min_by_key(|a| self.agent_loads.get(a).unwrap_or(&0))
                .cloned(),
            SchedulingStrategy::RoundRobin => {
                // Simplified, in reality would maintain an index
                available_agents.first().cloned()
            }
            _ => available_agents.first().cloned(),
        }
    }
}
