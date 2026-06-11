use crate::models::{AgentId, MissionId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Statistics for a single tool's usage during a mission.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolUsageStats {
    pub invocations: u32,
    pub successes: u32,
    pub failures: u32,
    pub total_latency_ms: u64,
}

impl ToolUsageStats {
    pub fn avg_latency_ms(&self) -> f64 {
        if self.invocations == 0 {
            0.0
        } else {
            self.total_latency_ms as f64 / self.invocations as f64
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.invocations == 0 {
            0.0
        } else {
            self.successes as f64 / self.invocations as f64
        }
    }
}

/// Effectiveness score for a single agent during a mission.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentEffectivenessScore {
    pub tasks_completed: u32,
    pub tasks_failed: u32,
    pub total_quality: f64,
    pub total_latency_ms: u64,
    pub task_count: u32,
}

impl AgentEffectivenessScore {
    pub fn avg_quality(&self) -> f64 {
        if self.task_count == 0 {
            0.0
        } else {
            self.total_quality / self.task_count as f64
        }
    }

    pub fn avg_latency_ms(&self) -> f64 {
        if self.task_count == 0 {
            0.0
        } else {
            self.total_latency_ms as f64 / self.task_count as f64
        }
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.tasks_completed + self.tasks_failed;
        if total == 0 {
            0.0
        } else {
            self.tasks_completed as f64 / total as f64
        }
    }
}

/// Comprehensive reflection data collected during mission execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionReflection {
    pub mission_id: MissionId,
    pub total_tasks: u32,
    pub completed_tasks: u32,
    pub failed_tasks: u32,
    pub retries: u32,
    pub tool_usage: HashMap<String, ToolUsageStats>,
    pub agent_effectiveness: HashMap<AgentId, AgentEffectivenessScore>,
    pub total_cost: f64,
    pub total_latency_ms: u64,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl MissionReflection {
    pub fn success_rate(&self) -> f64 {
        if self.total_tasks == 0 {
            0.0
        } else {
            self.completed_tasks as f64 / self.total_tasks as f64
        }
    }

    pub fn retry_ratio(&self) -> f64 {
        if self.total_tasks == 0 {
            0.0
        } else {
            self.retries as f64 / self.total_tasks as f64
        }
    }

    pub fn duration_secs(&self) -> f64 {
        self.total_latency_ms as f64 / 1000.0
    }
}

/// Tracks mission execution data and produces `MissionReflection` snapshots.
pub struct MissionReflector {
    reflections: HashMap<MissionId, MissionReflection>,
}

impl Default for MissionReflector {
    fn default() -> Self {
        Self::new()
    }
}

impl MissionReflector {
    pub fn new() -> Self {
        Self {
            reflections: HashMap::new(),
        }
    }

    /// Start tracking a new mission.
    pub fn start_mission(&mut self, mission_id: MissionId, total_tasks: u32) {
        let reflection = MissionReflection {
            mission_id,
            total_tasks,
            completed_tasks: 0,
            failed_tasks: 0,
            retries: 0,
            tool_usage: HashMap::new(),
            agent_effectiveness: HashMap::new(),
            total_cost: 0.0,
            total_latency_ms: 0,
            started_at: chrono::Utc::now(),
            completed_at: None,
        };
        self.reflections.insert(mission_id, reflection);
    }

    /// Record a tool usage event.
    pub fn track_tool_usage(
        &mut self,
        mission_id: &MissionId,
        tool: &str,
        success: bool,
        latency_ms: u64,
    ) {
        if let Some(reflection) = self.reflections.get_mut(mission_id) {
            let stats = reflection.tool_usage.entry(tool.to_string()).or_default();
            stats.invocations += 1;
            if success {
                stats.successes += 1;
            } else {
                stats.failures += 1;
            }
            stats.total_latency_ms += latency_ms;
        }
    }

    /// Record an agent task result.
    pub fn track_agent_result(
        &mut self,
        mission_id: &MissionId,
        agent_id: AgentId,
        success: bool,
        quality: f64,
        latency_ms: u64,
    ) {
        if let Some(reflection) = self.reflections.get_mut(mission_id) {
            if success {
                reflection.completed_tasks += 1;
            } else {
                reflection.failed_tasks += 1;
            }
            reflection.total_latency_ms += latency_ms;

            let score = reflection.agent_effectiveness.entry(agent_id).or_default();
            score.task_count += 1;
            if success {
                score.tasks_completed += 1;
            } else {
                score.tasks_failed += 1;
            }
            score.total_quality += quality;
            score.total_latency_ms += latency_ms;
        }
    }

    /// Record a retry event.
    pub fn track_retry(&mut self, mission_id: &MissionId) {
        if let Some(reflection) = self.reflections.get_mut(mission_id) {
            reflection.retries += 1;
        }
    }

    /// Record cost incurred.
    pub fn track_cost(&mut self, mission_id: &MissionId, cost: f64) {
        if let Some(reflection) = self.reflections.get_mut(mission_id) {
            reflection.total_cost += cost;
        }
    }

    /// Finalise and return the mission reflection.
    pub fn finalize(&mut self, mission_id: &MissionId) -> Option<MissionReflection> {
        if let Some(reflection) = self.reflections.get_mut(mission_id) {
            reflection.completed_at = Some(chrono::Utc::now());
            Some(reflection.clone())
        } else {
            None
        }
    }

    /// Generate a reflection report from mission data using the existing engine.
    pub fn reflect_on_mission(&self, reflection: &MissionReflection) -> super::ReflectionReport {
        let success = reflection.failed_tasks == 0 && reflection.completed_tasks > 0;
        let quality = if reflection.total_tasks > 0 {
            ((reflection.completed_tasks as f64 / reflection.total_tasks as f64) * 100.0) as u8
        } else {
            0
        };

        let mut report = super::ReflectionReport {
            mission_id: reflection.mission_id,
            task_id: None,
            agent_id: None,
            success,
            success_factors: Vec::new(),
            failure_reasons: Vec::new(),
            bottlenecks: Vec::new(),
            recommendations: Vec::new(),
            quality_score: quality,
        };

        if success {
            report
                .success_factors
                .push("All tasks completed successfully".into());
        }

        if reflection.failed_tasks > 0 {
            report.failure_reasons.push(format!(
                "{} tasks failed out of {}",
                reflection.failed_tasks, reflection.total_tasks
            ));
        }

        if reflection.retry_ratio() > 0.3 {
            report
                .bottlenecks
                .push("High retry ratio indicates instability".into());
            report
                .recommendations
                .push("Consider using a more reliable strategy".into());
        }

        // Identify poorly performing tools
        for (tool, stats) in &reflection.tool_usage {
            if stats.success_rate() < 0.5 && stats.invocations > 2 {
                report.bottlenecks.push(format!(
                    "Tool '{}' has low success rate ({:.0}%)",
                    tool,
                    stats.success_rate() * 100.0
                ));
            }
        }

        // Identify poorly performing agents
        for (agent_id, score) in &reflection.agent_effectiveness {
            if score.success_rate() < 0.5 && score.task_count > 1 {
                report.recommendations.push(format!(
                    "Agent {} has low effectiveness ({:.0}%) — consider replacement",
                    agent_id.0,
                    score.success_rate() * 100.0
                ));
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mission_id() -> MissionId {
        MissionId::new()
    }

    #[test]
    fn start_and_finalize_mission() {
        let mut reflector = MissionReflector::new();
        let mid = make_mission_id();
        reflector.start_mission(mid, 5);

        let reflection = reflector.finalize(&mid).unwrap();
        assert_eq!(reflection.total_tasks, 5);
        assert_eq!(reflection.completed_tasks, 0);
        assert!(reflection.completed_at.is_some());
    }

    #[test]
    fn track_tool_usage() {
        let mut reflector = MissionReflector::new();
        let mid = make_mission_id();
        reflector.start_mission(mid, 3);

        reflector.track_tool_usage(&mid, "grep", true, 50);
        reflector.track_tool_usage(&mid, "grep", false, 100);
        reflector.track_tool_usage(&mid, "grep", true, 75);

        let reflection = reflector.finalize(&mid).unwrap();
        let grep = &reflection.tool_usage["grep"];
        assert_eq!(grep.invocations, 3);
        assert_eq!(grep.successes, 2);
        assert_eq!(grep.failures, 1);
        assert!((grep.avg_latency_ms() - 75.0).abs() < f64::EPSILON);
    }

    #[test]
    fn track_agent_result() {
        let mut reflector = MissionReflector::new();
        let mid = make_mission_id();
        let agent = AgentId::new();
        reflector.start_mission(mid, 3);

        reflector.track_agent_result(&mid, agent, true, 0.9, 200);
        reflector.track_agent_result(&mid, agent, true, 0.8, 300);
        reflector.track_agent_result(&mid, agent, false, 0.3, 500);

        let reflection = reflector.finalize(&mid).unwrap();
        assert_eq!(reflection.completed_tasks, 2);
        assert_eq!(reflection.failed_tasks, 1);

        let eff = &reflection.agent_effectiveness[&agent];
        assert_eq!(eff.tasks_completed, 2);
        assert_eq!(eff.tasks_failed, 1);
        assert_eq!(eff.task_count, 3);
    }

    #[test]
    fn track_retry() {
        let mut reflector = MissionReflector::new();
        let mid = make_mission_id();
        reflector.start_mission(mid, 2);
        reflector.track_retry(&mid);
        reflector.track_retry(&mid);

        let reflection = reflector.finalize(&mid).unwrap();
        assert_eq!(reflection.retries, 2);
    }

    #[test]
    fn track_cost() {
        let mut reflector = MissionReflector::new();
        let mid = make_mission_id();
        reflector.start_mission(mid, 1);
        reflector.track_cost(&mid, 5.0);
        reflector.track_cost(&mid, 3.5);

        let reflection = reflector.finalize(&mid).unwrap();
        assert!((reflection.total_cost - 8.5).abs() < f64::EPSILON);
    }

    #[test]
    fn success_rate_calculation() {
        let reflection = MissionReflection {
            mission_id: make_mission_id(),
            total_tasks: 10,
            completed_tasks: 7,
            failed_tasks: 3,
            retries: 2,
            tool_usage: HashMap::new(),
            agent_effectiveness: HashMap::new(),
            total_cost: 0.0,
            total_latency_ms: 0,
            started_at: chrono::Utc::now(),
            completed_at: None,
        };
        assert!((reflection.success_rate() - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn retry_ratio_calculation() {
        let reflection = MissionReflection {
            mission_id: make_mission_id(),
            total_tasks: 4,
            completed_tasks: 4,
            failed_tasks: 0,
            retries: 2,
            tool_usage: HashMap::new(),
            agent_effectiveness: HashMap::new(),
            total_cost: 0.0,
            total_latency_ms: 0,
            started_at: chrono::Utc::now(),
            completed_at: None,
        };
        assert!((reflection.retry_ratio() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn reflect_on_successful_mission() {
        let reflector = MissionReflector::new();
        let reflection = MissionReflection {
            mission_id: make_mission_id(),
            total_tasks: 5,
            completed_tasks: 5,
            failed_tasks: 0,
            retries: 0,
            tool_usage: HashMap::new(),
            agent_effectiveness: HashMap::new(),
            total_cost: 10.0,
            total_latency_ms: 5000,
            started_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
        };

        let report = reflector.reflect_on_mission(&reflection);
        assert!(report.success);
        assert_eq!(report.quality_score, 100);
        assert!(!report.success_factors.is_empty());
    }

    #[test]
    fn reflect_on_failed_mission() {
        let reflector = MissionReflector::new();
        let reflection = MissionReflection {
            mission_id: make_mission_id(),
            total_tasks: 5,
            completed_tasks: 2,
            failed_tasks: 3,
            retries: 4,
            tool_usage: HashMap::new(),
            agent_effectiveness: HashMap::new(),
            total_cost: 50.0,
            total_latency_ms: 30000,
            started_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
        };

        let report = reflector.reflect_on_mission(&reflection);
        assert!(!report.success);
        assert!(!report.failure_reasons.is_empty());
        assert!(!report.bottlenecks.is_empty()); // high retry ratio
    }

    #[test]
    fn reflect_identifies_poor_tools() {
        let reflector = MissionReflector::new();
        let mut tool_usage = HashMap::new();
        tool_usage.insert(
            "flaky_tool".to_string(),
            ToolUsageStats {
                invocations: 10,
                successes: 2,
                failures: 8,
                total_latency_ms: 5000,
            },
        );

        let reflection = MissionReflection {
            mission_id: make_mission_id(),
            total_tasks: 5,
            completed_tasks: 5,
            failed_tasks: 0,
            retries: 0,
            tool_usage,
            agent_effectiveness: HashMap::new(),
            total_cost: 0.0,
            total_latency_ms: 0,
            started_at: chrono::Utc::now(),
            completed_at: None,
        };

        let report = reflector.reflect_on_mission(&reflection);
        assert!(report.bottlenecks.iter().any(|b| b.contains("flaky_tool")));
    }

    #[test]
    fn reflect_identifies_poor_agents() {
        let reflector = MissionReflector::new();
        let agent = AgentId::new();
        let mut agent_effectiveness = HashMap::new();
        agent_effectiveness.insert(
            agent,
            AgentEffectivenessScore {
                tasks_completed: 1,
                tasks_failed: 4,
                total_quality: 1.0,
                total_latency_ms: 10000,
                task_count: 5,
            },
        );

        let reflection = MissionReflection {
            mission_id: make_mission_id(),
            total_tasks: 5,
            completed_tasks: 1,
            failed_tasks: 4,
            retries: 0,
            tool_usage: HashMap::new(),
            agent_effectiveness,
            total_cost: 0.0,
            total_latency_ms: 0,
            started_at: chrono::Utc::now(),
            completed_at: None,
        };

        let report = reflector.reflect_on_mission(&reflection);
        assert!(report
            .recommendations
            .iter()
            .any(|r| r.contains("replacement")));
    }

    #[test]
    fn finalize_unknown_mission_returns_none() {
        let mut reflector = MissionReflector::new();
        assert!(reflector.finalize(&make_mission_id()).is_none());
    }

    #[test]
    fn tool_usage_stats_empty() {
        let stats = ToolUsageStats::default();
        assert!((stats.avg_latency_ms() - 0.0).abs() < f64::EPSILON);
        assert!((stats.success_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn agent_effectiveness_empty() {
        let score = AgentEffectivenessScore::default();
        assert!((score.avg_quality() - 0.0).abs() < f64::EPSILON);
        assert!((score.success_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn duration_secs_conversion() {
        let reflection = MissionReflection {
            mission_id: make_mission_id(),
            total_tasks: 1,
            completed_tasks: 1,
            failed_tasks: 0,
            retries: 0,
            tool_usage: HashMap::new(),
            agent_effectiveness: HashMap::new(),
            total_cost: 0.0,
            total_latency_ms: 5500,
            started_at: chrono::Utc::now(),
            completed_at: None,
        };
        assert!((reflection.duration_secs() - 5.5).abs() < f64::EPSILON);
    }

    #[test]
    fn default_reflector() {
        let mut r = MissionReflector::default();
        assert!(r.finalize(&make_mission_id()).is_none());
    }

    #[test]
    fn track_operations_on_unknown_mission_are_noop() {
        let mut reflector = MissionReflector::new();
        let mid = make_mission_id();
        // These should not panic
        reflector.track_tool_usage(&mid, "test", true, 10);
        reflector.track_agent_result(&mid, AgentId::new(), true, 0.9, 100);
        reflector.track_retry(&mid);
        reflector.track_cost(&mid, 5.0);
    }
}
